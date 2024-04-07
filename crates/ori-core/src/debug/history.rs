use std::collections::VecDeque;

use crate::{event::Event, window::WindowId};
use instant::{Duration, Instant};

/// Emitted when a window builds it's view tree.
#[derive(Clone, Debug)]
pub struct BuildItem {
    /// The start time of the build.
    pub start: Instant,
    /// The duration of the build.
    pub duration: Duration,
    /// The window that built the view tree.
    pub window: WindowId,
}

impl BuildItem {
    /// Create a new build item.
    pub fn new(start: Instant, window: WindowId) -> Self {
        Self {
            start,
            duration: start.elapsed(),
            window,
        }
    }
}

/// Emitted when a window rebuilds it's view tree.
#[derive(Clone, Debug)]
pub struct RebuildItem {
    /// The start time of the rebuild.
    pub start: Instant,
    /// The duration of the rebuild.
    pub duration: Duration,
    /// The window that rebuilt the view tree.
    pub window: WindowId,
}

impl RebuildItem {
    /// Create a new rebuild item.
    pub fn new(start: Instant, window: WindowId) -> Self {
        Self {
            start,
            duration: start.elapsed(),
            window,
        }
    }
}

/// Emitted when a window handles an event.
#[derive(Clone, Debug)]
pub struct EventItem {
    /// The start time of the event.
    pub start: Instant,
    /// The duration of the event.
    pub duration: Duration,
    /// The window that handled the event.
    pub window: WindowId,
    /// The event that was handled.
    pub name: &'static str,
}

impl EventItem {
    /// Create a new event item.
    pub fn new(start: Instant, window: WindowId, event: &Event) -> Self {
        Self {
            start,
            duration: start.elapsed(),
            window,
            name: event.name(),
        }
    }
}

/// Emitted when a window lays out it's view tree.
#[derive(Clone, Debug)]
pub struct LayoutItem {
    /// The start time of the layout.
    pub start: Instant,
    /// The duration of the layout.
    pub duration: Duration,
    /// The window that laid out the view tree.
    pub window: WindowId,
}

impl LayoutItem {
    /// Create a new layout item.
    pub fn new(start: Instant, window: WindowId) -> Self {
        Self {
            start,
            duration: start.elapsed(),
            window,
        }
    }
}

/// Emitted when a window draws it's view tree.
#[derive(Clone, Debug)]
pub struct DrawItem {
    /// The start time of the draw.
    pub start: Instant,
    /// The duration of the draw.
    pub duration: Duration,
    /// The window that drew the view tree.
    pub window: WindowId,
}

impl DrawItem {
    /// Create a new draw item.
    pub fn new(start: Instant, window: WindowId) -> Self {
        Self {
            start,
            duration: start.elapsed(),
            window,
        }
    }
}

/// An item in the history of the application.
#[derive(Clone, Debug)]
pub enum HistoryItem {
    /// A window builds it's view tree.
    Build(BuildItem),
    /// A window rebuilds it's view tree.
    Rebuild(RebuildItem),
    /// A window handles an event.
    Event(EventItem),
    /// A window lays out it's view tree.
    Layout(LayoutItem),
    /// A window draws it's view tree.
    Draw(DrawItem),
}

impl HistoryItem {
    /// Get the window that the item is associated with, if any.
    pub fn window(&self) -> Option<WindowId> {
        Some(match self {
            Self::Build(item) => item.window,
            Self::Rebuild(item) => item.window,
            Self::Event(item) => item.window,
            Self::Layout(item) => item.window,
            Self::Draw(item) => item.window,
        })
    }

    /// Get the start time of the item.
    pub fn start(&self) -> Instant {
        match self {
            Self::Build(item) => item.start,
            Self::Rebuild(item) => item.start,
            Self::Event(item) => item.start,
            Self::Layout(item) => item.start,
            Self::Draw(item) => item.start,
        }
    }

    /// Get the duration of the item, if any.
    pub fn duration(&self) -> Option<Duration> {
        Some(match self {
            Self::Build(item) => item.duration,
            Self::Rebuild(item) => item.duration,
            Self::Event(item) => item.duration,
            Self::Layout(item) => item.duration,
            Self::Draw(item) => item.duration,
        })
    }
}

impl From<BuildItem> for HistoryItem {
    fn from(item: BuildItem) -> Self {
        Self::Build(item)
    }
}

impl From<RebuildItem> for HistoryItem {
    fn from(item: RebuildItem) -> Self {
        Self::Rebuild(item)
    }
}

impl From<EventItem> for HistoryItem {
    fn from(item: EventItem) -> Self {
        Self::Event(item)
    }
}

impl From<LayoutItem> for HistoryItem {
    fn from(item: LayoutItem) -> Self {
        Self::Layout(item)
    }
}

impl From<DrawItem> for HistoryItem {
    fn from(item: DrawItem) -> Self {
        Self::Draw(item)
    }
}

/// A history of events that have occurred in the application.
#[derive(Clone, Debug, Default)]
pub struct History {
    items: VecDeque<HistoryItem>,
    max: Option<usize>,
}

impl History {
    /// Create a new history.
    pub fn new() -> Self {
        Self {
            items: VecDeque::new(),
            max: None,
        }
    }

    /// Create a new history with a maximum number of items.
    pub fn with_max_items(max_items: usize) -> Self {
        Self {
            items: VecDeque::new(),
            max: Some(max_items),
        }
    }

    /// Get the maximum number of items in the history.
    pub fn max_items(&self) -> Option<usize> {
        self.max
    }

    /// Get the number of items in the history.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Get whether the history is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Clear the history.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Add an item to the history.
    pub fn push(&mut self, item: impl Into<HistoryItem>) {
        if let Some(max_items) = self.max {
            if self.items.len() >= max_items {
                self.items.pop_front();
            }
        }

        self.items.push_back(item.into());
    }

    /// Get an iterator over the items in the history.
    pub fn items(&self) -> impl Iterator<Item = &HistoryItem> {
        self.items.iter()
    }

    /// Get the average build time of the history.
    pub fn average_build_time(&self) -> Option<Duration> {
        let mut total = Duration::ZERO;
        let mut count = 0;

        for item in self.items.iter() {
            if let HistoryItem::Build(item) = item {
                total += item.duration;
                count += 1;
            }
        }

        if count > 0 {
            Some(total / count)
        } else {
            None
        }
    }

    /// Get the average rebuild time of the history.
    pub fn average_rebuild_time(&self) -> Option<Duration> {
        let mut total = Duration::ZERO;
        let mut count = 0;

        for item in self.items.iter() {
            if let HistoryItem::Rebuild(item) = item {
                total += item.duration;
                count += 1;
            }
        }

        if count > 0 {
            Some(total / count)
        } else {
            None
        }
    }

    /// Get the average event time of the history.
    pub fn average_event_time(&self) -> Option<Duration> {
        let mut total = Duration::ZERO;
        let mut count = 0;

        for item in self.items.iter() {
            if let HistoryItem::Event(item) = item {
                total += item.duration;
                count += 1;
            }
        }

        if count > 0 {
            Some(total / count)
        } else {
            None
        }
    }

    /// Get the average layout time of the history.
    pub fn average_layout_time(&self) -> Option<Duration> {
        let mut total = Duration::ZERO;
        let mut count = 0;

        for item in self.items.iter() {
            if let HistoryItem::Layout(item) = item {
                total += item.duration;
                count += 1;
            }
        }

        if count > 0 {
            Some(total / count)
        } else {
            None
        }
    }

    /// Get the average draw time of the history.
    pub fn average_draw_time(&self) -> Option<Duration> {
        let mut total = Duration::ZERO;
        let mut count = 0;

        for item in self.items.iter() {
            if let HistoryItem::Draw(item) = item {
                total += item.duration;
                count += 1;
            }
        }

        if count > 0 {
            Some(total / count)
        } else {
            None
        }
    }
}
