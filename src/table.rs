//! Native text tables.
//!
//! A table takes its position from GPUI layout. AppKit virtualizes and draws
//! the static text rows and the scroll edge effect.

use std::sync::Arc;

use gpui::{
    App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement, LayoutId,
    Pixels, Refineable, Rgba, Style, StyleRefinement, Styled, Window,
};

pub use gpui::FontWeight;

const DEFAULT_ROW_HEIGHT: f32 = 44.0;
const DEFAULT_FONT_SIZE: f32 = 13.0;

/// Defines one static row in a native text table.
#[derive(Clone, Debug, PartialEq)]
pub struct NativeTextTableRow {
    pub(crate) text: String,
    pub(crate) background_color: Rgba,
    pub(crate) foreground_color: Rgba,
}

impl NativeTextTableRow {
    /// Creates a row with text, background color, and foreground color.
    pub fn new(text: impl Into<String>, background_color: Rgba, foreground_color: Rgba) -> Self {
        Self {
            text: text.into(),
            background_color,
            foreground_color,
        }
    }

    /// Returns the row text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the row background color.
    pub fn background_color(&self) -> Rgba {
        self.background_color
    }

    /// Returns the row foreground color.
    pub fn foreground_color(&self) -> Rgba {
        self.foreground_color
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextTableConfiguration {
    pub(crate) rows: Arc<[NativeTextTableRow]>,
    pub(crate) row_height: Pixels,
    pub(crate) font_size: Pixels,
    pub(crate) font_weight: FontWeight,
    pub(crate) initial_scroll_offset: Pixels,
}

impl TextTableConfiguration {
    fn new(rows: Arc<[NativeTextTableRow]>) -> Self {
        Self {
            rows,
            row_height: Pixels::from(DEFAULT_ROW_HEIGHT),
            font_size: Pixels::from(DEFAULT_FONT_SIZE),
            font_weight: FontWeight::NORMAL,
            initial_scroll_offset: Pixels::from(0.0),
        }
    }
}

/// Places a native, virtualized text table in GPUI layout.
///
/// AppKit uses `NSTableView` inside `NSScrollView` to draw and scroll the rows.
///
/// The rows are static for the life of the native table. A changed builder
/// configuration replaces the AppKit table during synchronization.
///
/// # Panics
///
/// Painting panics unless [`crate::install`] initialized the application and
/// [`crate::NativeRoot`] wraps this window's root. It also panics off the main
/// thread or when AppKit fails to create the scroll view.
pub struct NativeTextTable {
    id: String,
    configuration: TextTableConfiguration,
    style: StyleRefinement,
}

/// Creates a native text table with static rows.
pub fn native_text_table(
    id: impl Into<String>,
    rows: impl Into<Arc<[NativeTextTableRow]>>,
) -> NativeTextTable {
    NativeTextTable {
        id: id.into(),
        configuration: TextTableConfiguration::new(rows.into()),
        style: StyleRefinement::default(),
    }
}

impl NativeTextTable {
    /// Sets the height of each row in logical pixels.
    pub fn row_height(mut self, height: Pixels) -> Self {
        self.configuration.row_height = height;
        self
    }

    /// Sets the row text size in logical pixels.
    pub fn font_size(mut self, size: Pixels) -> Self {
        self.configuration.font_size = size;
        self
    }

    /// Sets the GPUI font weight used for all rows.
    pub fn font_weight(mut self, weight: FontWeight) -> Self {
        self.configuration.font_weight = weight;
        self
    }

    /// Sets the exact initial vertical scroll offset in logical pixels.
    ///
    /// AppKit clamps the offset to the scrollable content range.
    pub fn initial_scroll_offset(mut self, offset: Pixels) -> Self {
        self.configuration.initial_scroll_offset = offset;
        self
    }
}

impl IntoElement for NativeTextTable {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Styled for NativeTextTable {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl Element for NativeTextTable {
    type RequestLayoutState = Style;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone().into())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.refine(&self.style);
        let layout_id = window.request_layout(style.clone(), [], cx);
        (layout_id, style)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) {
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        style: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        style.paint(bounds, window, cx, |_, _| {});

        let window_id = window.window_handle().window_id();
        let mtm = crate::lifecycle::main_thread_marker(cx);
        crate::native_views::record_text_table(
            window_id,
            window,
            TextTableFrame {
                id: self.id.clone(),
                configuration: self.configuration.clone(),
                bounds,
            },
            mtm,
            cx,
        )
        .unwrap_or_else(|error| panic!("the text table must apply: {error}"));
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextTableFrame {
    pub(crate) id: String,
    pub(crate) configuration: TextTableConfiguration,
    pub(crate) bounds: Bounds<Pixels>,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use gpui::{FontWeight, px, rgba};

    use super::{NativeTextTableRow, native_text_table};

    #[test]
    fn text_row_keeps_caller_content_and_colors() {
        let background = rgba(0x123456ff);
        let foreground = rgba(0xfedcbaff);
        let row = NativeTextTableRow::new("Status", background, foreground);

        assert_eq!(row.text(), "Status");
        assert_eq!(row.background_color(), background);
        assert_eq!(row.foreground_color(), foreground);
    }

    #[test]
    fn table_builder_keeps_rows_and_text_configuration() {
        let rows: Arc<[NativeTextTableRow]> = Arc::from([
            NativeTextTableRow::new("One", rgba(0x112233ff), rgba(0xffffffff)),
            NativeTextTableRow::new("Two", rgba(0x445566ff), rgba(0x000000ff)),
        ]);
        let table = native_text_table("content", rows.clone())
            .row_height(px(56.0))
            .font_size(px(22.0))
            .font_weight(FontWeight::SEMIBOLD)
            .initial_scroll_offset(px(4480.5));

        assert_eq!(table.id, "content");
        assert_eq!(table.configuration.rows, rows);
        assert_eq!(table.configuration.row_height, px(56.0));
        assert_eq!(table.configuration.font_size, px(22.0));
        assert_eq!(table.configuration.font_weight, FontWeight::SEMIBOLD);
        assert_eq!(table.configuration.initial_scroll_offset, px(4480.5));
    }
}
