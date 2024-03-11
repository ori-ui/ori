use std::{
    any::type_name,
    hash::{Hash, Hasher},
};

use crate::layout::{Affine, Rect};

/// A debug tree.
#[derive(Clone, Debug, Hash)]
pub struct DebugTree {
    name: Option<&'static str>,
    content: Vec<Option<DebugTree>>,
    rect: Option<Rect>,
    transform: Option<Affine>,
}

impl Default for DebugTree {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugTree {
    /// Create a new unknown debug tree.
    pub fn new() -> Self {
        Self {
            name: None,
            content: Vec::new(),
            rect: None,
            transform: None,
        }
    }

    /// Set the name.
    pub fn set_type<T: ?Sized>(&mut self) {
        if self.name.is_none() {
            self.name = Some(type_name::<T>());
        }
    }

    /// Set the name.
    pub fn set_name(&mut self, name: &'static str) {
        self.name = Some(name);
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

    fn short_name_recurse(&self, name: &'static str) -> String {
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
            Some(name) => self.short_name_recurse(name),
            None => "unknown".to_string(),
        }
    }

    /// Set the layout.
    pub fn set_rect(&mut self, rect: Rect) {
        self.rect = Some(rect);
    }

    /// Add a child.
    pub fn rect(&self) -> Rect {
        self.rect.unwrap_or_default()
    }

    /// Set the transform.
    pub fn set_transform(&mut self, transform: Affine) {
        self.transform = Some(transform);
    }

    /// Get the transform.
    pub fn transform(&self) -> Affine {
        self.transform.unwrap_or_default()
    }

    /// Insert a child.
    pub fn insert(&mut self, index: usize, child: DebugTree) {
        if index >= self.content.len() {
            self.content.resize_with(index + 1, || None);
        }

        self.content[index] = Some(child);
    }

    /// Remove a child.
    pub fn remove(&mut self, index: usize) -> Option<DebugTree> {
        self.content.get_mut(index).and_then(Option::take)
    }

    /// Remove a child or create a new one.
    pub fn remove_or_new(&mut self, index: usize) -> DebugTree {
        self.remove(index).unwrap_or_default()
    }

    /// Truncate the number of children.
    pub fn truncate(&mut self, len: usize) {
        self.content.truncate(len);
    }

    /// Get the child at the given index.
    pub fn get_child(&self, index: usize) -> Option<&DebugTree> {
        self.content.get(index).and_then(Option::as_ref)
    }

    /// Get the child at the given index.
    pub fn get_child_mut(&mut self, index: usize) -> Option<&mut DebugTree> {
        self.content.get_mut(index).and_then(Option::as_mut)
    }

    /// Get the number of children.
    pub fn content(&self) -> impl Iterator<Item = &DebugTree> {
        self.content.iter().filter_map(Option::as_ref)
    }
}
