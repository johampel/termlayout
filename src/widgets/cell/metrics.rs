use crate::widgets::CellAnchor;
use crate::{Dimension, LayoutOptions, Rect, WrapMode};
use std::cmp::max;

/// Helper struct to calculate the metrics of a cell.
///
/// This contains metrics for the cell as a whole and the content of the cell. Typically, the content
/// dimension is different from the cell's dimension, so when it comes to rendering, the content
/// needs additional clipping or padding.
///
/// # Fields
/// - `padding`: The padding around the content within the cell
pub(crate) struct CellMetrics {
    cell_dim: Dimension,
    content_dim: Dimension,
    content_clip_rect: Rect,
    pub(crate) padding: (usize, usize),
}

impl CellMetrics {
    /// Creates a new [`CellMetrics`] struct.
    ///
    /// # Parameters
    /// - `options`: The layout options for the cell
    /// - `content_dim`: The layout dimension for the content
    /// - `content_clip`: The clipping rect for the content. If this is `None`, the content is
    ///   not clipped. If this is `Some(rect)`, the content is clipped to the given rect.
    /// - `cell_anchor`: The anchor for the cell content
    ///
    /// # Returns
    /// A new [`CellMetrics`] instance with calculated dimensions and positions
    pub(crate) fn new(
        options: &LayoutOptions,
        content_dim: Dimension,
        content_clip: Option<Rect>,
        cell_anchor: CellAnchor,
    ) -> Self {
        let content = content_clip.map_or_else(
            || content_dim.into(),
            |clip| clip.intersect(content_dim.into()),
        );
        let anchor_factors = cell_anchor.factors();

        // Calculate the overall dimension, and the dimension/placement of the interesting rects
        // related to the overall dimension.
        let overall_dim = Dimension::new(
            max(content.dim.width, options.dim.width),
            max(content.dim.height, options.dim.height),
        );
        let cell_rect = Rect::new(
            (overall_dim.width.saturating_sub(options.dim.width) * anchor_factors.0) / 2,
            (overall_dim.height.saturating_sub(options.dim.height) * anchor_factors.1) / 2,
            options.dim,
        );
        let cell_visible_rect = options.visible_rect().with_offset(cell_rect.x, cell_rect.y);
        let content_rect = Rect::new(
            (overall_dim.width.saturating_sub(content.dim.width) * anchor_factors.0) / 2,
            (overall_dim.height.saturating_sub(content.dim.height) * anchor_factors.1) / 2,
            content.dim,
        );
        let content_visible_rect = content_rect.intersect_relative(cell_visible_rect);

        // Calculate fields
        let cell_dim = cell_visible_rect.dim;
        let padding = (
            content_rect.x.saturating_sub(cell_visible_rect.x),
            content_rect.y.saturating_sub(cell_visible_rect.y),
        );
        let content_clip_rect = Rect::new(
            content.x + content_visible_rect.x,
            content.y + content_visible_rect.y,
            content_visible_rect.dim,
        );

        Self {
            cell_dim,
            content_dim,
            content_clip_rect,
            padding,
        }
    }

    /// Returns the [`LayoutOptions`] for the entire cell.
    ///
    /// # Parameters
    /// - `fill_rows`: The `fill_rows` flag for the options
    /// - `wrap_mode`: The `wrap_mode` for the options
    ///
    /// # Returns
    /// The [`LayoutOptions`] for the cell
    pub(crate) fn cell_options(&self, fill_rows: bool, wrap_mode: WrapMode) -> LayoutOptions {
        LayoutOptions::new(self.cell_dim, fill_rows, wrap_mode, None)
    }

    /// Returns the [`LayoutOptions`] for the  cell content.
    ///
    /// # Parameters
    /// - `fill_rows`: The `fill_rows` flag for the options
    /// - `wrap_mode`: The `wrap_mode` for the options
    ///
    /// # Returns
    /// The [`LayoutOptions`] for the content
    pub(crate) fn content_options(&self, fill_rows: bool, wrap_mode: WrapMode) -> LayoutOptions {
        LayoutOptions::new(
            self.content_dim,
            fill_rows,
            wrap_mode,
            Some(self.content_clip_rect),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LayoutOptions;

    #[test]
    fn cell_metrics_content_inside_cell_no_clip() {
        let content_dim = Dimension::new(200, 200);
        let content_clip = Some(Rect::new(101, 102, Dimension::new(10, 10)));

        // Exact fit
        let options = LayoutOptions::default().with_dim(Dimension::new(10, 10));
        let metrics = CellMetrics::new(&options, content_dim, content_clip, CellAnchor::Center);
        assert_eq!(metrics.cell_dim, Dimension::new(10, 10));
        assert_eq!(metrics.padding, (0, 0));
        assert_eq!(metrics.content_dim, content_dim);
        assert_eq!(
            metrics.content_clip_rect,
            Rect::new(101, 102, Dimension::new(10, 10))
        );

        // With Padding
        let options = LayoutOptions::default().with_dim(Dimension::new(20, 20));
        let metrics = CellMetrics::new(&options, content_dim, content_clip, CellAnchor::Center);
        assert_eq!(metrics.cell_dim, Dimension::new(20, 20));
        assert_eq!(metrics.padding, (5, 5));
        assert_eq!(metrics.content_dim, content_dim);
        assert_eq!(
            metrics.content_clip_rect,
            Rect::new(101, 102, Dimension::new(10, 10))
        );
    }

    #[test]
    fn cell_metrics_content_inside_cell_with_clip() {
        let content_dim = Dimension::new(200, 200);
        let content_clip = Some(Rect::new(101, 102, Dimension::new(10, 10)));

        // Exact fit
        let options = LayoutOptions::default()
            .with_clip(Some(Rect::new(3, 4, Dimension::new(4, 5))))
            .with_dim(Dimension::new(10, 10));
        let metrics = CellMetrics::new(&options, content_dim, content_clip, CellAnchor::Center);
        assert_eq!(metrics.cell_dim, Dimension::new(4, 5));
        assert_eq!(metrics.padding, (0, 0));
        assert_eq!(
            metrics.content_clip_rect,
            Rect::new(104, 106, Dimension::new(4, 5))
        );

        // With Padding (case clip something from top)
        let options = LayoutOptions::default()
            .with_dim(Dimension::new(20, 20))
            .with_clip(Some(Rect::new(3, 6, Dimension::new(4, 5))));
        let metrics = CellMetrics::new(&options, content_dim, content_clip, CellAnchor::Center);
        assert_eq!(metrics.cell_dim, Dimension::new(4, 5));
        assert_eq!(metrics.padding, (2, 0));
        assert_eq!(
            metrics.content_clip_rect,
            Rect::new(101, 103, Dimension::new(2, 5))
        );

        // With Padding (case clip something from bottom)
        let options = LayoutOptions::default()
            .with_dim(Dimension::new(20, 20))
            .with_clip(Some(Rect::new(3, 12, Dimension::new(4, 5))));
        let metrics = CellMetrics::new(&options, content_dim, content_clip, CellAnchor::Center);
        assert_eq!(metrics.cell_dim, Dimension::new(4, 5));
        assert_eq!(metrics.padding, (2, 0));
        assert_eq!(
            metrics.content_clip_rect,
            Rect::new(101, 109, Dimension::new(2, 3))
        );
    }

    #[test]
    fn cell_metrics_cell_inside_content_no_clip() {
        let content_dim = Dimension::new(200, 200);
        let content_clip = Some(Rect::new(101, 102, Dimension::new(20, 20)));

        let options = LayoutOptions::default().with_dim(Dimension::new(10, 10));
        let metrics = CellMetrics::new(&options, content_dim, content_clip, CellAnchor::Center);
        assert_eq!(metrics.cell_dim, Dimension::new(10, 10));
        assert_eq!(metrics.padding, (0, 0));
        assert_eq!(
            metrics.content_clip_rect,
            Rect::new(106, 107, Dimension::new(10, 10))
        );
    }

    #[test]
    fn cell_metrics_cell_inside_content_with_clip() {
        let content_dim = Dimension::new(200, 200);
        let content_clip = Some(Rect::new(101, 102, Dimension::new(20, 20)));

        let options = LayoutOptions::default()
            .with_dim(Dimension::new(10, 10))
            .with_clip(Some(Rect::new(3, 4, Dimension::new(4, 5))));
        let metrics = CellMetrics::new(&options, content_dim, content_clip, CellAnchor::Center);
        assert_eq!(metrics.cell_dim, Dimension::new(4, 5));
        assert_eq!(metrics.padding, (0, 0));
        assert_eq!(
            metrics.content_clip_rect,
            Rect::new(109, 111, Dimension::new(4, 5))
        );
    }
}
