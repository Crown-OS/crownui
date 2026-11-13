use std::{
    collections::VecDeque,
    hash::{Hash, Hasher},
};

use geometry::Size;
use graphics::{TextLayout, TextLayoutId, TextStyle, TextSystem};
use rustc_hash::{FxHashMap, FxHasher};

const WIDTH_QUANTUM: f32 = 0.5;
const DEFAULT_BUDGET_BYTES: usize = 8 * 1024 * 1024;
const EVICTION_HEADROOM_DIVISOR: usize = 4;
/// Frames a shaped layout outlives the last node that drew it, so text that comes back right
/// away (a counter's digits moving between labels, a row scrolled back in) is not shaped again.
const RETAINED_FRAMES: u64 = 2;

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
    /// The id the renderer knows the layout by while some node draws it.
    id: Option<TextLayoutId>,
    last_used: u64,
    bytes: usize,
}

/// Shapes text through a [`TextSystem`] and caches results by content, style and width, so
/// remounted rows and repeated strings never reshape. A layout no node draws anymore is kept for
/// [`RETAINED_FRAMES`] frames, then only its metrics; entries no node draws are evicted
/// least-recently-used once the byte budget is exceeded.
pub struct TextCache<T: TextSystem> {
    system: T,
    entries: FxHashMap<TextKey, Entry<T::Layout>>,
    references: FxHashMap<TextLayoutId, (TextKey, usize)>,
    uploads: Vec<(TextLayoutId, T::Layout)>,
    releases: Vec<TextLayoutId>,
    /// Entries whose last drawing node let go, with the frame it happened in.
    unreferenced: VecDeque<(u64, TextKey)>,
    frame: u64,
    next_id: u64,
    clock: u64,
    bytes: usize,
    budget: usize,
}

fn hash_text(text: &str) -> u64 {
    let mut hasher = FxHasher::default();
    text.hash(&mut hasher);
    hasher.finish()
}

impl<T: TextSystem> TextCache<T> {
    /// Wraps a text system with the default 8 MiB budget.
    pub fn new(system: T) -> Self {
        Self {
            system,
            entries: FxHashMap::default(),
            references: FxHashMap::default(),
            uploads: Vec::new(),
            releases: Vec::new(),
            unreferenced: VecDeque::new(),
            frame: 0,
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
                last_used: self.clock,
                bytes,
            },
        );
        self.unreferenced.push_back((self.frame, key));
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

    /// The drawable layout for a node broken at `width`, or unbroken for `None`: shares ids
    /// between nodes with identical text and width.
    pub fn acquire(
        &mut self,
        text: &str,
        style: &TextStyle,
        width: Option<f32>,
    ) -> (TextLayoutId, TextMetrics) {
        let (key, metrics) = self.entry(text, style, width);
        let needs_reshape = self
            .entries
            .get(&key)
            .is_some_and(|entry| entry.layout.is_none());
        if needs_reshape {
            let layout = self.system.layout(
                text,
                style,
                key.width_steps.map(|steps| steps as f32 * WIDTH_QUANTUM),
            );
            if let Some(entry) = self.entries.get_mut(&key) {
                let bytes = layout.memory_bytes();
                entry.bytes += bytes;
                self.bytes += bytes;
                entry.layout = Some(layout);
            }
        }
        let Some(entry) = self.entries.get_mut(&key) else {
            return (TextLayoutId(0), metrics);
        };
        let id = match entry.id {
            Some(id) => id,
            None => {
                let id = TextLayoutId(self.next_id);
                self.next_id += 1;
                entry.id = Some(id);
                if let Some(layout) = &entry.layout {
                    self.uploads.push((id, layout.clone()));
                }
                id
            }
        };
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
                self.unreferenced.push_back((self.frame, key));
            }
        }
    }

    /// Ends a frame: layouts no node has drawn for [`RETAINED_FRAMES`] frames, measured only or
    /// let go, are dropped, keeping their metrics.
    pub fn finish_frame(&mut self) {
        self.frame += 1;
        while let Some(&(released, key)) = self.unreferenced.front() {
            if self.frame - released < RETAINED_FRAMES {
                break;
            }
            self.unreferenced.pop_front();
            if let Some(entry) = self.entries.get_mut(&key)
                && entry.id.is_none()
                && let Some(layout) = entry.layout.take()
            {
                let bytes = layout.memory_bytes();
                entry.bytes = entry.bytes.saturating_sub(bytes);
                self.bytes = self.bytes.saturating_sub(bytes);
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

    /// Over budget, drops the least recently used unreferenced entries until a quarter of the
    /// budget is free, so a stream of new strings pays for one scan per many insertions.
    fn evict(&mut self) {
        if self.bytes <= self.budget {
            return;
        }
        let target = self.budget - self.budget / EVICTION_HEADROOM_DIVISOR;
        let mut candidates: Vec<(u64, TextKey)> = self
            .entries
            .iter()
            .filter(|(_, entry)| entry.id.is_none())
            .map(|(key, entry)| (entry.last_used, *key))
            .collect();
        candidates.sort_unstable_by_key(|(last_used, _)| *last_used);
        for (_, key) in candidates {
            if self.bytes <= target {
                break;
            }
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
        let (first, _) = cache.acquire("Hello", &style, Some(100.0));
        let (second, _) = cache.acquire("Hello", &style, Some(99.8));
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
        let (id, _) = cache.acquire("Row", &style, Some(50.0));
        cache.acquire("Row", &style, Some(50.0));
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

    #[test]
    fn a_released_layout_is_kept_for_two_frames_then_dropped() {
        let mut cache = TextCache::new(RecordingTextSystem);
        let style = TextStyle::default();
        let (id, _) = cache.acquire("42", &style, None);
        cache.release(id);
        let has_layout = |cache: &TextCache<RecordingTextSystem>| {
            cache.entries.values().all(|entry| entry.layout.is_some())
        };
        cache.finish_frame();
        assert!(has_layout(&cache));
        cache.finish_frame();
        assert!(!has_layout(&cache));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn a_layout_only_measured_is_dropped_after_two_frames_with_its_bytes() {
        let mut cache = TextCache::new(RecordingTextSystem);
        let style = TextStyle::default();
        cache.measure("A wrapping description", &style, Some(80.0));
        let measured = cache.bytes();
        cache.finish_frame();
        cache.finish_frame();
        assert!(cache.entries.values().all(|entry| entry.layout.is_none()));
        assert!(cache.bytes() < measured);
        let (id, _) = cache.acquire("A wrapping description", &style, Some(80.0));
        assert_eq!(cache.bytes(), measured);
        assert_eq!(cache.take_uploads().len(), 1);
        cache.release(id);
    }

    #[test]
    fn unbroken_text_draws_the_layout_it_was_measured_with() {
        let mut cache = TextCache::new(RecordingTextSystem);
        let style = TextStyle::default();
        cache.measure("Label", &style, None);
        cache.acquire("Label", &style, None);
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.take_uploads().len(), 1);
    }
}
