use crate::config::AnimationConfig;

const CELL_FACTOR: f32 = 1.5;
const WOBBLE: f32 = 6.0;
const LABEL_COLUMNS: usize = 20;
const LABEL_ROW_HEIGHT: f32 = 18.0;
const LABEL_GAP: f32 = 8.0;

/// Where each animated box sits and how it moves; the same formulas in every framework.
pub struct BoxGrid {
    pub box_size: f32,
    cell: f32,
    columns: usize,
    rows: usize,
}

impl BoxGrid {
    pub fn new(config: AnimationConfig) -> Self {
        let cell = config.box_size * CELL_FACTOR;
        let columns = ((config.width / cell) as usize).max(1);
        Self {
            box_size: config.box_size,
            cell,
            columns,
            rows: config.boxes.div_ceil(columns),
        }
    }

    pub fn position(&self, index: usize) -> (f32, f32) {
        let inset = (self.cell - self.box_size) / 2.0;
        (
            (index % self.columns) as f32 * self.cell + inset,
            (index / self.columns) as f32 * self.cell + inset,
        )
    }

    pub fn height(&self) -> f32 {
        self.rows as f32 * self.cell
    }

    pub fn offset(index: usize, seconds: f32) -> (f32, f32) {
        let phase = seconds * 2.0 + index as f32 * 0.15;
        (WOBBLE * phase.sin(), WOBBLE * phase.cos())
    }

    pub fn rotation(index: usize, seconds: f32) -> f32 {
        (seconds * 90.0 + index as f32 * 3.0) % 360.0
    }
}

/// Where each per-frame label sits, below the boxes, and what it shows.
pub struct LabelGrid {
    width: f32,
    top: f32,
}

impl LabelGrid {
    pub fn new(config: AnimationConfig, boxes: &BoxGrid) -> Self {
        Self {
            width: config.width / LABEL_COLUMNS as f32,
            top: boxes.height() + LABEL_GAP,
        }
    }

    pub fn position(&self, index: usize) -> (f32, f32) {
        (
            (index % LABEL_COLUMNS) as f32 * self.width,
            self.top + (index / LABEL_COLUMNS) as f32 * LABEL_ROW_HEIGHT,
        )
    }

    pub fn text(index: usize, seconds: f32) -> String {
        ((seconds * 1000.0) as u64 + index as u64).to_string()
    }
}
