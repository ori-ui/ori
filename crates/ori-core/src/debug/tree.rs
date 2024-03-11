use std::{
    any::type_name,
    hash::{Hash, Hasher},
    mem,
    time::Duration,
};

use crate::layout::{Affine, Rect, Space};

/// Debug information about how a debug tree was laid out.
#[derive(Clone, Debug, Default)]
pub struct DebugLayout {
    /// The space.
    pub space: Space,
    /// The flex.
    pub flex: f32,
    /// Whether the flex is tight.
    pub tight: bool,
}

impl Hash for DebugLayout {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.space.hash(state);
        self.flex.to_bits().hash(state);
        self.tight.hash(state);
    }
}

/// Debug information about how a debug tree was drawn.
#[derive(Clone, Debug, Default)]
pub struct DebugDraw {
    /// The rect.
    pub rect: Rect,
    /// The transform.
    pub transform: Affine,
    /// The depth.
    pub depth: f32,
}

impl Hash for DebugDraw {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rect.hash(state);
        self.transform.hash(state);
        self.depth.to_bits().hash(state);
    }
}

/// A debug tree.
#[derive(Clone, Debug)]
pub struct DebugTree {
    name: Option<&'static str>,
    content: Vec<DebugTree>,
    layout: Option<DebugLayout>,
    draw: Option<DebugDraw>,
    decay: bool,

    build_time: Option<Duration>,
    rebuild_time: Option<Duration>,
    event_time: Option<Duration>,
    layout_time: Option<Duration>,
    draw_time: Option<Duration>,
}

impl Default for DebugTree {
    fn default() -> Self {
        Self::new()
    }
}

impl Hash for DebugTree {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.content.hash(state);
        self.layout.hash(state);
        self.draw.hash(state);
    }
}

impl DebugTree {
    /// Create a new unknown debug tree.
    pub fn new() -> Self {
        Self {
            name: None,
            content: Vec::new(),
            layout: None,
            draw: None,
            decay: false,

            build_time: None,
            rebuild_time: None,
            event_time: None,
            layout_time: None,
            draw_time: None,
        }
    }

    /// Set the name.
    pub fn set_type<T: ?Sized>(&mut self) {
        self.name = Some(type_name::<T>());
        self.decay = false;
    }

    /// Set the name.
    pub fn set_name(&mut self, name: &'static str) {
        self.name = Some(name);
        self.decay = false;
    }

    /// Get the name.
    pub fn name(&self) -> &'static str {
        self.name.unwrap_or("unknown")
    }

    /// Get a fast hash of the tree.
    pub fn fast_hash(&self) -> u64 {
        let mut hasher = seahash::SeaHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn collapse_name(name: &str) -> String {
        let special = [' ', '<', '>', '(', ')', '[', ']', ',', ';'];

        let mut collapsed = String::new();

        let mut start = 0;
        let mut end = 0;

        for c in name.chars() {
            if special.contains(&c) {
                collapsed.push_str(&name[start..end]);

                end += c.len_utf8();
                start = end;

                collapsed.push(c);

                continue;
            }

            if c == ':' {
                end += c.len_utf8();
                start = end;

                continue;
            }

            end += c.len_utf8();
        }

        collapsed.push_str(&name[start..]);
        collapsed
    }

    fn short_name_internal(&self, name: &'static str) -> String {
        const MAX_NAME_LEN: usize = 64;

        let mut ranges = Vec::new();

        for child in self.content() {
            if let Some(child_name) = child.name {
                for (index, _) in name.match_indices(child_name) {
                    ranges.push((index, child_name.len()));
                }
            }
        }

        ranges.sort_by_key(|(index, _)| *index);

        let mut replaced = String::new();
        let mut last = 0;

        for (index, len) in ranges {
            if index > last {
                replaced.push_str(&name[last..index]);
                replaced.push('_');
            }

            last = last.max(index + len);
        }

        replaced.push_str(&name[last..]);

        let mut collapsed = Self::collapse_name(&replaced);

        if collapsed.len() > MAX_NAME_LEN {
            collapsed.truncate(MAX_NAME_LEN - 3);
            collapsed.push_str("...");
        }

        collapsed
    }

    /// Get the short name.
    pub fn short_name(&self) -> String {
        match self.name {
            Some(name) => self.short_name_internal(name),
            None => "unknown".to_string(),
        }
    }

    /// Set the layout.
    pub fn set_layout(&mut self, layout: DebugLayout) {
        self.layout = Some(layout);
    }

    /// Get the layout.
    pub fn layout(&self) -> DebugLayout {
        self.layout.clone().unwrap_or_default()
    }

    /// Set the draw.
    pub fn set_draw(&mut self, draw: DebugDraw) {
        self.draw = Some(draw);
    }

    /// Get the draw.
    pub fn draw(&self) -> DebugDraw {
        self.draw.clone().unwrap_or_default()
    }

    /// Insert a child.
    pub fn insert(&mut self, index: usize, child: DebugTree) {
        if index >= self.content.len() {
            self.content.resize_with(index + 1, Self::default);
        }

        self.content[index] = child;
    }

    /// Remove a child.
    pub fn remove(&mut self, index: usize) -> Option<DebugTree> {
        self.get_child_mut(index).map(mem::take)
    }

    /// Remove a child or create a new one.
    pub fn remove_or_new(&mut self, index: usize) -> DebugTree {
        self.remove(index).unwrap_or_default()
    }

    /// Truncate the number of children.
    pub fn truncate(&mut self, len: usize) {
        self.content.truncate(len);
    }

    /// Get a tree at the given path.
    pub fn get_path(&self, path: &[usize]) -> Option<&DebugTree> {
        let mut tree = self;

        for &index in path {
            tree = tree.get_child(index)?;
        }

        Some(tree)
    }

    /// Get the child at the given index.
    pub fn get_child(&self, index: usize) -> Option<&DebugTree> {
        self.content.get(index)
    }

    /// Get the child at the given index.
    pub fn get_child_mut(&mut self, index: usize) -> Option<&mut DebugTree> {
        self.content.get_mut(index)
    }

    /// Get the number of children.
    pub fn content(&self) -> &[DebugTree] {
        &self.content
    }

    /// Decay the tree, pruning any stale branches.
    pub fn decay(&mut self) {
        self.decay = true;

        self.content.retain_mut(|child| {
            if child.decay {
                false
            } else {
                child.decay();
                true
            }
        });
    }
}

impl DebugTree {
    /// Set the build time.
    pub fn set_build_time(&mut self, time: Duration) {
        self.build_time = Some(time);
    }

    /// Get the build time.
    pub fn build_time(&self) -> Option<Duration> {
        self.build_time
    }

    /// Set the rebuild time.
    pub fn set_rebuild_time(&mut self, time: Duration) {
        self.rebuild_time = Some(time);
    }

    /// Get the rebuild time.
    pub fn rebuild_time(&self) -> Option<Duration> {
        self.rebuild_time
    }

    /// Set the event time.
    pub fn set_event_time(&mut self, time: Duration) {
        self.event_time = Some(time);
    }

    /// Get the event time.
    pub fn event_time(&self) -> Option<Duration> {
        self.event_time
    }

    /// Set the layout time.
    pub fn set_layout_time(&mut self, time: Duration) {
        self.layout_time = Some(time);
    }

    /// Get the layout time.
    pub fn layout_time(&self) -> Option<Duration> {
        self.layout_time
    }

    /// Set the draw time.
    pub fn set_draw_time(&mut self, time: Duration) {
        self.draw_time = Some(time);
    }

    /// Get the draw time.
    pub fn draw_time(&self) -> Option<Duration> {
        self.draw_time
    }
}
