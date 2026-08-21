//! Builds native text tables with AppKit.

use std::collections::{HashMap, HashSet};

use gpui::{FontWeight, Rgba, Window, WindowId};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{
    NSColor, NSControlTextEditingDelegate, NSFont, NSScrollView, NSTableColumn, NSTableView,
    NSTableViewDataSource, NSTableViewDelegate, NSTableViewStyle, NSTextField, NSView,
    NSWindowOrderingMode,
};
use objc2_foundation::{NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString};

use crate::table::{TextTableConfiguration, TextTableFrame};

use super::handle::{NativeWindowHandle, appkit_frame};

#[derive(Debug)]
pub(crate) struct TableSourceIvars {
    configuration: TextTableConfiguration,
}

define_class!(
    /// Supplies owned static rows to the virtualized table on the main thread.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "LNGpuiTableSource"]
    #[ivars = TableSourceIvars]
    #[derive(Debug)]
    pub(crate) struct TableSource;

    unsafe impl NSObjectProtocol for TableSource {}

    unsafe impl NSTableViewDataSource for TableSource {
        #[unsafe(method(numberOfRowsInTableView:))]
        fn number_of_rows(&self, _table: &NSTableView) -> isize {
            isize::try_from(self.ivars().configuration.rows.len()).unwrap_or(isize::MAX)
        }
    }

    unsafe impl NSControlTextEditingDelegate for TableSource {}

    unsafe impl NSTableViewDelegate for TableSource {
        #[unsafe(method_id(tableView:viewForTableColumn:row:))]
        fn view_for_row(
            &self,
            _table: &NSTableView,
            _column: Option<&NSTableColumn>,
            row: isize,
        ) -> Option<Retained<NSView>> {
            let configuration = &self.ivars().configuration;
            let row = usize::try_from(row)
                .ok()
                .and_then(|index| configuration.rows.get(index));
            row.map(|row| {
                let mtm = MainThreadMarker::from(self);
                let text = NSString::from_str(&row.text);
                let label = NSTextField::labelWithString(&text, mtm);
                label.setDrawsBackground(true);
                label.setBackgroundColor(Some(&ns_color(row.background_color)));
                label.setTextColor(Some(&ns_color(row.foreground_color)));
                label.setFont(Some(&NSFont::systemFontOfSize_weight(
                    positive_dimension(f64::from(configuration.font_size)),
                    appkit_font_weight(configuration.font_weight),
                )));
                Retained::into_super(Retained::into_super(label))
            })
        }
    }
);

impl TableSource {
    fn new(configuration: TextTableConfiguration, mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(TableSourceIvars { configuration });
        // SAFETY: the ivars are initialized before NSObject initialization.
        unsafe { msg_send![super(this), init] }
    }
}

/// Owns the text tables of each window.
#[derive(Default)]
pub(crate) struct NativeTextTableRegistry {
    windows: HashMap<WindowId, NativeTextTableWindow>,
}

impl NativeTextTableRegistry {
    /// Releases the tables of a closed window.
    pub(crate) fn remove_window(&mut self, window_id: WindowId) {
        self.windows.remove(&window_id);
    }

    /// Prepares the window that holds the tables.
    pub(crate) fn ensure_window(
        &mut self,
        window_id: WindowId,
        gpui_window: &Window,
        mtm: MainThreadMarker,
    ) -> Result<(), String> {
        if let std::collections::hash_map::Entry::Vacant(entry) = self.windows.entry(window_id) {
            entry.insert(NativeTextTableWindow::new(gpui_window, mtm)?);
        }
        Ok(())
    }

    /// Applies the recorded bounds to the native views.
    pub(crate) fn flush(
        &mut self,
        window_id: WindowId,
        gpui_window: &mut Window,
        frames: Vec<TextTableFrame>,
        mtm: MainThreadMarker,
    ) -> Result<(), String> {
        let Some(native) = self.windows.get_mut(&window_id) else {
            return Ok(());
        };
        native.flush(gpui_window, frames, mtm)
    }
}

/// Holds the text tables of one window.
struct NativeTextTableWindow {
    handle: NativeWindowHandle,
    tables: HashMap<String, NativeTextTable>,
}

impl NativeTextTableWindow {
    fn new(gpui_window: &Window, mtm: MainThreadMarker) -> Result<Self, String> {
        Ok(Self {
            handle: NativeWindowHandle::acquire(gpui_window, mtm)?,
            tables: HashMap::new(),
        })
    }

    fn flush(
        &mut self,
        gpui_window: &mut Window,
        frames: Vec<TextTableFrame>,
        mtm: MainThreadMarker,
    ) -> Result<(), String> {
        let Some(gpui_frame) = self.handle.gpui_frame_in_superview(gpui_window)? else {
            gpui_window.refresh();
            return Ok(());
        };
        let live_tables: HashSet<_> = frames.iter().map(|table| table.id.clone()).collect();

        for table in frames {
            let frame = appkit_frame(table.bounds, gpui_frame);
            match self.tables.entry(table.id) {
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    if entry.get().configuration != table.configuration {
                        entry.insert(NativeTextTable::new(
                            &self.handle,
                            table.configuration,
                            frame,
                            mtm,
                        ));
                    } else {
                        entry.get().sync_frame(frame, mtm);
                    }
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(NativeTextTable::new(
                        &self.handle,
                        table.configuration,
                        frame,
                        mtm,
                    ));
                }
            }
        }
        self.tables.retain(|id, _| live_tables.contains(id));
        Ok(())
    }
}

/// Holds one `NSScrollView` and its virtualized `NSTableView`.
struct NativeTextTable {
    scroll_view: Retained<NSScrollView>,
    table: Retained<NSTableView>,
    column: Retained<NSTableColumn>,
    _source: Retained<TableSource>,
    content_height: f64,
    configuration: TextTableConfiguration,
}

impl NativeTextTable {
    fn new(
        handle: &NativeWindowHandle,
        configuration: TextTableConfiguration,
        frame: NSRect,
        mtm: MainThreadMarker,
    ) -> Self {
        let row_height = positive_dimension(f64::from(configuration.row_height));
        let content_height = table_content_height(configuration.rows.len(), row_height);
        let table_frame = local_table_frame(frame.size.width, content_height);
        let table = NSTableView::initWithFrame(NSTableView::alloc(mtm), table_frame);
        let column = NSTableColumn::initWithIdentifier(
            NSTableColumn::alloc(mtm),
            &NSString::from_str("LN.GPUI.TableColumn"),
        );
        column.setWidth(table_frame.size.width);
        table.addTableColumn(&column);
        table.setRowHeight(row_height);
        table.setIntercellSpacing(NSSize::new(0.0, 0.0));
        table.setHeaderView(None);
        table.setStyle(NSTableViewStyle::Plain);

        let source = TableSource::new(configuration.clone(), mtm);
        // SAFETY: the native text table owns `source`, so it outlives the NSTableView.
        unsafe {
            table.setDataSource(Some(ProtocolObject::from_ref(&*source)));
            table.setDelegate(Some(ProtocolObject::from_ref(&*source)));
        }
        table.reloadData();
        table.setFrame(table_frame);

        let scroll_view = NSScrollView::initWithFrame(NSScrollView::alloc(mtm), frame);
        scroll_view.setHasVerticalScroller(true);
        scroll_view.setAutomaticallyAdjustsContentInsets(true);
        scroll_view.setDocumentView(Some(&table));
        scroll_view.setFrame(frame);

        let host = handle.superview();
        host.addSubview_positioned_relativeTo(
            &scroll_view,
            NSWindowOrderingMode::Above,
            Some(handle.gpui_view()),
        );

        sync_column_width(&scroll_view, &table, &column, content_height);
        set_initial_scroll_offset(
            &scroll_view,
            f64::from(configuration.initial_scroll_offset),
            content_height,
        );

        Self {
            scroll_view,
            table,
            column,
            _source: source,
            content_height,
            configuration,
        }
    }

    /// Synchronizes AppKit geometry on the main thread.
    fn sync_frame(&self, frame: NSRect, _mtm: MainThreadMarker) {
        self.scroll_view.setFrame(frame);
        sync_column_width(
            &self.scroll_view,
            &self.table,
            &self.column,
            self.content_height,
        );
    }
}

impl Drop for NativeTextTable {
    fn drop(&mut self) {
        debug_assert!(
            MainThreadMarker::new().is_some(),
            "the text table releases on the main thread"
        );
        self.scroll_view.removeFromSuperview();
    }
}

fn ns_color(color: Rgba) -> Retained<NSColor> {
    NSColor::colorWithSRGBRed_green_blue_alpha(
        color.r.into(),
        color.g.into(),
        color.b.into(),
        color.a.into(),
    )
}

fn positive_dimension(value: f64) -> f64 {
    if value.is_finite() {
        value.max(1.0)
    } else {
        1.0
    }
}

fn table_content_height(row_count: usize, row_height: f64) -> f64 {
    row_count as f64 * positive_dimension(row_height)
}

fn local_table_frame(width: f64, content_height: f64) -> NSRect {
    NSRect::new(
        NSPoint::new(0.0, 0.0),
        NSSize::new(positive_dimension(width), content_height.max(0.0)),
    )
}

fn clamped_scroll_offset(requested: f64, content_height: f64, viewport_height: f64) -> f64 {
    if !requested.is_finite() {
        return 0.0;
    }
    let maximum = (content_height - viewport_height).max(0.0);
    requested.clamp(0.0, maximum)
}

fn set_initial_scroll_offset(scroll_view: &NSScrollView, requested: f64, content_height: f64) {
    let clip_view = scroll_view.contentView();
    let offset = clamped_scroll_offset(requested, content_height, clip_view.bounds().size.height);
    clip_view.scrollToPoint(NSPoint::new(0.0, offset));
    scroll_view.reflectScrolledClipView(&clip_view);
}

fn sync_column_width(
    scroll_view: &NSScrollView,
    table: &NSTableView,
    column: &NSTableColumn,
    content_height: f64,
) {
    let width = positive_dimension(scroll_view.contentView().bounds().size.width);
    column.setWidth(width);
    table.setFrame(local_table_frame(width, content_height));
}

fn appkit_font_weight(weight: FontWeight) -> f64 {
    const WEIGHTS: [(f32, f64); 9] = [
        (100.0, -0.8),
        (200.0, -0.6),
        (300.0, -0.4),
        (400.0, 0.0),
        (500.0, 0.23),
        (600.0, 0.3),
        (700.0, 0.4),
        (800.0, 0.56),
        (900.0, 0.62),
    ];

    let value = if weight.0.is_finite() {
        weight.0.clamp(WEIGHTS[0].0, WEIGHTS[WEIGHTS.len() - 1].0)
    } else {
        FontWeight::NORMAL.0
    };
    for pair in WEIGHTS.windows(2) {
        let [(low_weight, low_value), (high_weight, high_value)] = pair else {
            unreachable!("weight windows have two entries")
        };
        if value <= *high_weight {
            let fraction = (value - low_weight) / (high_weight - low_weight);
            return low_value + (high_value - low_value) * f64::from(fraction);
        }
    }
    WEIGHTS[WEIGHTS.len() - 1].1
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use gpui::{Bounds, FontWeight, point, px, size};
    use objc2_foundation::{NSPoint, NSRect, NSSize};

    use super::{
        appkit_font_weight, clamped_scroll_offset, local_table_frame, table_content_height,
    };
    use crate::platform::mac::handle::appkit_frame;

    fn assert_rect(actual: NSRect, expected: NSRect) {
        assert!((actual.origin.x - expected.origin.x).abs() < f64::EPSILON);
        assert!((actual.origin.y - expected.origin.y).abs() < f64::EPSILON);
        assert!((actual.size.width - expected.size.width).abs() < f64::EPSILON);
        assert!((actual.size.height - expected.size.height).abs() < f64::EPSILON);
    }

    #[test]
    fn places_text_table_under_full_size_title_bar() {
        let bounds: Bounds<_> = Bounds::new(point(px(276.0), px(0.0)), size(px(820.0), px(700.0)));
        let gpui_frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1120.0, 700.0));

        assert_rect(
            appkit_frame(bounds, gpui_frame),
            NSRect::new(NSPoint::new(276.0, 0.0), NSSize::new(820.0, 700.0)),
        );
    }

    #[test]
    fn places_text_table_from_inset_gpui_origin() {
        let bounds: Bounds<_> = Bounds::new(point(px(18.0), px(24.0)), size(px(420.0), px(300.0)));
        let gpui_frame = NSRect::new(NSPoint::new(8.0, 32.0), NSSize::new(600.0, 568.0));

        assert_rect(
            appkit_frame(bounds, gpui_frame),
            NSRect::new(NSPoint::new(26.0, 276.0), NSSize::new(420.0, 300.0)),
        );
    }

    #[test]
    fn table_geometry_uses_table_width_and_row_height() {
        let height = table_content_height(200, 56.0);

        assert_eq!(height, 11_200.0);
        assert_rect(
            local_table_frame(820.0, height),
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(820.0, 11_200.0)),
        );
    }

    #[test]
    fn initial_offset_preserves_pixels_and_clamps_to_content() {
        assert_eq!(clamped_scroll_offset(4480.5, 11_200.0, 700.0), 4480.5);
        assert_eq!(clamped_scroll_offset(-24.0, 11_200.0, 700.0), 0.0);
        assert_eq!(clamped_scroll_offset(20_000.0, 11_200.0, 700.0), 10_500.0);
        assert_eq!(clamped_scroll_offset(50.0, 200.0, 700.0), 0.0);
    }

    #[test]
    fn gpui_semibold_maps_to_the_appkit_semibold_weight() {
        assert!((appkit_font_weight(FontWeight::SEMIBOLD) - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn reconciliation_drops_absent_text_tables() {
        let mut tables = HashMap::from([("gone".to_owned(), 1), ("live".to_owned(), 2)]);
        let live = HashSet::from(["live".to_owned()]);

        tables.retain(|id, _| live.contains(id));

        assert_eq!(tables, HashMap::from([("live".to_owned(), 2)]));
    }
}
