use std::ops::Range;

use geometry::Size;

/// How an index-based lazy list places its items: rows of equal height, optionally split into
/// as many columns of at least `min_width` as fit (CSS `repeat(auto-fill, minmax(w, 1fr))`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemLayout {
    /// Narrowest a column may be; `0` gives one column as wide as the list.
    pub min_width: f32,
    /// Height of every item.
    pub height: f32,
    /// Space between columns and between rows.
    pub gap: (f32, f32),
    /// Space around all items, inside the scrolled content.
    pub padding: f32,
}

impl ItemLayout {
    /// One full-width item per row, each `height` tall.
    pub const fn rows(height: f32) -> Self {
        Self {
            min_width: 0.0,
            height,
            gap: (0.0, 0.0),
            padding: 0.0,
        }
    }

    /// Cells at least `cell.width` wide and `cell.height` tall, `gap` apart (between columns,
    /// between rows); spare width is shared out so the cells fill each row.
    pub const fn cells(cell: Size, gap: (f32, f32)) -> Self {
        Self {
            min_width: cell.width,
            height: cell.height,
            gap,
            padding: 0.0,
        }
    }

    /// The same layout with `padding` around the items.
    pub const fn with_padding(self, padding: f32) -> Self {
        Self { padding, ..self }
    }

    fn inner_width(&self, width: f32) -> f32 {
        (width - 2.0 * self.padding).max(0.0)
    }

    /// How many columns fit in `width`; at least one.
    pub fn columns(&self, width: f32) -> usize {
        if self.min_width <= 0.0 {
            return 1;
        }
        let pitch = self.min_width + self.gap.0;
        (((self.inner_width(width) + self.gap.0) / pitch).floor() as usize).max(1)
    }

    /// Width of each of `columns` columns in `width`.
    pub fn column_width(&self, width: f32, columns: usize) -> f32 {
        let gaps = self.gap.0 * columns.saturating_sub(1) as f32;
        ((self.inner_width(width) - gaps) / columns.max(1) as f32).max(0.0)
    }

    /// Distance from one row's top to the next.
    pub fn stride(&self) -> f32 {
        self.height + self.gap.1
    }

    /// Rows `count` items fill in `columns` columns.
    pub fn row_count(count: usize, columns: usize) -> usize {
        count.div_ceil(columns.max(1))
    }

    /// Top of `row` in the scrolled content.
    pub fn row_top(&self, row: usize) -> f32 {
        self.padding + row as f32 * self.stride()
    }

    /// Height of the scrolled content holding `rows` rows.
    pub fn content_height(&self, rows: usize) -> f32 {
        let gaps = self.gap.1 * rows.saturating_sub(1) as f32;
        2.0 * self.padding + rows as f32 * self.height + gaps
    }

    /// Rows that intersect `extent` pixels from `offset`, widened by `overscan` rows each way.
    pub fn visible_rows(
        &self,
        offset: f32,
        extent: f32,
        rows: usize,
        overscan: usize,
    ) -> Range<usize> {
        let stride = self.stride();
        if stride <= 0.0 || rows == 0 {
            return 0..0;
        }
        let top = (offset - self.padding).max(0.0);
        let bottom = (offset + extent.max(0.0) - self.padding).max(0.0);
        let first = (top / stride).floor() as usize;
        let last = (bottom / stride).ceil() as usize;
        first.saturating_sub(overscan).min(rows)..(last + overscan).min(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GRID: ItemLayout =
        ItemLayout::cells(Size::new(90.0, 100.0), (6.0, 2.0)).with_padding(12.0);

    #[test]
    fn as_many_columns_fit_as_css_auto_fill_would_place() {
        assert_eq!(GRID.columns(24.0 + 90.0), 1);
        assert_eq!(GRID.columns(24.0 + 90.0 * 2.0 + 6.0), 2);
        assert_eq!(GRID.columns(24.0 + 90.0 * 2.0 + 5.0), 1);
        assert_eq!(GRID.columns(0.0), 1);
        assert_eq!(ItemLayout::rows(30.0).columns(1000.0), 1);
    }

    #[test]
    fn spare_width_is_shared_by_the_columns() {
        let width = 24.0 + 2.0 * 90.0 + 6.0 + 40.0;
        assert_eq!(GRID.column_width(width, 2), 110.0);
        assert_eq!(ItemLayout::rows(30.0).column_width(300.0, 1), 300.0);
    }

    #[test]
    fn the_content_holds_every_row_and_the_padding() {
        assert_eq!(ItemLayout::row_count(10, 4), 3);
        assert_eq!(GRID.content_height(3), 24.0 + 300.0 + 4.0);
        assert_eq!(GRID.content_height(0), 24.0);
        assert_eq!(GRID.row_top(2), 12.0 + 204.0);
    }

    #[test]
    fn visible_rows_include_overscan_and_stay_in_bounds() {
        let list = ItemLayout::rows(20.0);
        assert_eq!(list.visible_rows(0.0, 100.0, 1000, 4), 0..9);
        assert_eq!(list.visible_rows(1000.0, 100.0, 1000, 4), 46..59);
        assert_eq!(list.visible_rows(19_990.0, 100.0, 1000, 4), 995..1000);
        assert_eq!(list.visible_rows(0.0, 100.0, 0, 4), 0..0);
        assert_eq!(GRID.visible_rows(0.0, 50.0, 10, 0), 0..1);
    }
}
