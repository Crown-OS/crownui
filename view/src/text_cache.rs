use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use geometry::Size;
use graphics::{TextLayout, TextLayoutId, TextStyle, TextSystem};

const WIDTH_QUANTUM: f32 = 0.5;
const DEFAULT_BUDGET_BYTES: usize = 8 * 1024 * 1024;

/// Measurements of a shaped text, enough for layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextMetrics {
    /// Laid-out size.
    pub size: Size,
    /// CSS min-content width.
    pub min_width: f32,
    /// CSS max-content width.
    pub max_width: f32,
    /// First baseline.
    pub baseline: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TextKey {
    text: u64,
    style: u64,
    width_steps: Option<u32>,
}

struct Entry<L> {
    metrics: TextMetrics,
    layout: Option<L>,
    id: Option<TextLayoutId>,
    uploaded: bool,
    last_used: u64,
    bytes: usize,
}

/// Shapes text through a [`TextSystem`] and caches results by content, style and width, so
/// remounted rows and repeated strings never reshape. Measurement-only entries are evicted
/// least-recently-used once the byte budget is exceeded.
pub struct TextCache<T: TextSystem> {
    system: T,
    entries: HashMap<TextKey, Entry<T::Layout>>,
    references: HashMap<TextLayoutId, (TextKey, usize)>,
    uploads: Vec<(TextLayoutId, T::Layout)>,
    releases: Vec<TextLayoutId>,
    next_id: u64,
    clock: u64,
    bytes: usize,
    budget: usize,
}

fn hash_text(text: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

impl<T: TextSystem> TextCache<T> {
    /// Wraps a text system with the default 8 MiB budget.
    pub fn new(system: T) -> Self {
        Self {
            system,
            entries: HashMap::new(),
            references: HashMap::new(),
            uploads: Vec::new(),
            releases: Vec::new(),
            next_id: 1,
            clock: 0,
            bytes: 0,
            budget: DEFAULT_BUDGET_BYTES,
        }
    }

    /// The wrapped text system, e.g. to register fonts.
    pub fn system_mut(&mut self) -> &mut T {
        &mut self.system
    }

    /// Changes the byte budget for cached layouts.
    pub fn set_budget(&mut self, bytes: usize) {
        self.budget = bytes;
        self.evict();
    }

    fn key(text: &str, style: &TextStyle, max_width: Option<f32>) -> TextKey {
        TextKey {
            text: hash_text(text),
            style: style.shaping_key(),
            width_steps: max_width
                .filter(|width| width.is_finite())
                .map(|width| (width.max(0.0) / WIDTH_QUANTUM).ceil() as u32),
        }
    }

    fn entry(
        &mut self,
        text: &str,
        style: &TextStyle,
        max_width: Option<f32>,
    ) -> (TextKey, TextMetrics) {
        self.clock += 1;
        let key = Self::key(text, style, max_width);
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.last_used = self.clock;
            return (key, entry.metrics);
        }
        let width = key.width_steps.map(|steps| steps as f32 * WIDTH_QUANTUM);
        let layout = self.system.layout(text, style, width);
        let metrics = TextMetrics {
            size: layout.size(),
            min_width: layout.min_intrinsic_width(),
            max_width: layout.max_intrinsic_width(),
            baseline: layout.first_baseline(),
        };
        let bytes = layout.memory_bytes() + text.len();
        self.bytes += bytes;
        self.entries.insert(
            key,
            Entry {
                metrics,
                layout: Some(layout),
                id: None,
                uploaded: false,
                last_used: self.clock,
                bytes,
            },
        );
        self.evict();
        (key, metrics)
    }

    /// Measures `text` broken at `max_width`.
    pub fn measure(
        &mut self,
        text: &str,
        style: &TextStyle,
        max_width: Option<f32>,
    ) -> TextMetrics {
        self.entry(text, style, max_width).1
    }

    /// The drawable layout for a node: shares ids between nodes with identical text and width.
    pub fn acquire(
        &mut self,
        text: &str,
        style: &TextStyle,
        width: f32,
    ) -> (TextLayoutId, TextMetrics) {
        let (key, metrics) = self.entry(text, style, Some(width));
        let needs_reshape = self
            .entries
            .get(&key)
            .is_some_and(|entry| entry.layout.is_none() && !entry.uploaded);
        if needs_reshape {
            let layout = self.system.layout(
                text,
                style,
                key.width_steps.map(|steps| steps as f32 * WIDTH_QUANTUM),
            );
            if let Some(entry) = self.entries.get_mut(&key) {
                entry.layout = Some(layout);
            }
        }
        let Some(entry) = self.entries.get_mut(&key) else {
            return (TextLayoutId(0), metrics);
        };
        let id = *entry.id.get_or_insert_with(|| {
            let id = TextLayoutId(self.next_id);
            self.next_id += 1;
            id
        });
        if !entry.uploaded
            && let Some(layout) = entry.layout.take()
        {
            self.uploads.push((id, layout));
            entry.uploaded = true;
        }
        self.references.entry(id).or_insert((key, 0)).1 += 1;
        (id, metrics)
    }

    /// Drops one node's reference to `id`; the renderer releases it when none remain.
    pub fn release(&mut self, id: TextLayoutId) {
        let Some((key, count)) = self.references.get_mut(&id) else {
            return;
        };
        *count = count.saturating_sub(1);
        if *count == 0 {
            let key = *key;
            self.references.remove(&id);
            self.releases.push(id);
            if let Some(entry) = self.entries.get_mut(&key) {
                entry.id = None;
                entry.uploaded = false;
            }
        }
    }

    /// Layouts the renderer must receive.
    pub fn take_uploads(&mut self) -> Vec<(TextLayoutId, T::Layout)> {
        std::mem::take(&mut self.uploads)
    }

    /// Layouts the renderer may forget.
    pub fn take_releases(&mut self) -> Vec<TextLayoutId> {
        std::mem::take(&mut self.releases)
    }

    /// Number of cached entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Bytes held by cached layouts.
    pub fn bytes(&self) -> usize {
        self.bytes
    }

    fn evict(&mut self) {
        while self.bytes > self.budget {
            let candidate = self
                .entries
                .iter()
                .filter(|(_, entry)| entry.id.is_none())
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(key, _)| *key);
            let Some(key) = candidate else {
                break;
            };
            if let Some(entry) = self.entries.remove(&key) {
                self.bytes = self.bytes.saturating_sub(entry.bytes);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use graphics::RecordingTextSystem;

    use super::*;

    #[test]
    fn identical_text_shares_one_layout() {
        let mut cache = TextCache::new(RecordingTextSystem);
        let style = TextStyle::default();
        let (first, _) = cache.acquire("Hello", &style, 100.0);
        let (second, _) = cache.acquire("Hello", &style, 99.8);
        assert_eq!(first, second);
        assert_eq!(cache.take_uploads().len(), 1);
    }

    #[test]
    fn widths_quantize_upward_so_text_never_wraps_early() {
        let style = TextStyle::default();
        let key = TextCache::<RecordingTextSystem>::key("Ada", &style, Some(26.09));
        assert_eq!(key.width_steps, Some(53));
    }

    #[test]
    fn layouts_are_released_with_their_last_reference() {
        let mut cache = TextCache::new(RecordingTextSystem);
        let style = TextStyle::default();
        let (id, _) = cache.acquire("Row", &style, 50.0);
        cache.acquire("Row", &style, 50.0);
        cache.release(id);
        assert!(cache.take_releases().is_empty());
        cache.release(id);
        assert_eq!(cache.take_releases(), vec![id]);
    }

    #[test]
    fn measurement_entries_are_evicted_over_budget() {
        let mut cache = TextCache::new(RecordingTextSystem);
        cache.set_budget(64);
        let style = TextStyle::default();
        for index in 0..50 {
            cache.measure(&format!("text number {index}"), &style, None);
        }
        assert!(cache.bytes() <= 64 || cache.len() == 1);
    }
}
