//! Reads the AppKit views of a GPUI window.

use gpui::{Bounds, Pixels};
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{NSView, NSWindow};
use objc2_foundation::{NSPoint, NSRect, NSSize};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

/// Holds the AppKit views of one window.
pub(super) struct NativeWindowHandle {
    gpui_view: Retained<NSView>,
    superview: Retained<NSView>,
    window: Retained<NSWindow>,
}

impl NativeWindowHandle {
    /// Reads the views from a GPUI window.
    pub(super) fn acquire(
        gpui_window: &gpui::Window,
        _mtm: MainThreadMarker,
    ) -> Result<Self, String> {
        let raw = HasWindowHandle::window_handle(gpui_window)
            .map_err(|error| format!("the window handle must exist: {error}"))?
            .as_raw();
        let appkit = match raw {
            RawWindowHandle::AppKit(handle) => handle,
            other => return Err(format!("the window handle must be AppKit: {other:?}")),
        };

        // SAFETY: the handle holds a live NSView, and the retain keeps it alive
        // after the handle expires.
        let gpui_view = unsafe {
            Retained::retain(appkit.ns_view.as_ptr().cast::<NSView>())
                .ok_or("the view pointer must be valid")?
        };
        let window = gpui_view.window().ok_or("the view must have a window")?;
        // SAFETY: the view stays alive during this call.
        let superview = unsafe { gpui_view.superview() }.ok_or("the view must have a superview")?;

        Ok(Self {
            gpui_view,
            superview,
            window,
        })
    }

    /// Returns the view that GPUI draws into.
    pub(super) fn gpui_view(&self) -> &NSView {
        &self.gpui_view
    }

    /// Returns the parent of the GPUI view.
    pub(super) fn superview(&self) -> &NSView {
        &self.superview
    }

    /// Returns the window.
    pub(super) fn window(&self) -> &NSWindow {
        &self.window
    }

    /// Returns an owned reference to the GPUI view.
    pub(super) fn retained_gpui_view(&self) -> Retained<NSView> {
        self.gpui_view.clone()
    }

    /// Returns the frame of the GPUI view in its parent.
    ///
    /// The result is `None` while the GPUI viewport and the view size differ.
    pub(super) fn gpui_frame_in_superview(
        &self,
        gpui_window: &gpui::Window,
    ) -> Result<Option<NSRect>, String> {
        let native_bounds = self.gpui_view.bounds();
        let native_frame = self.gpui_view.frame();
        let viewport = gpui_window.viewport_size();
        let viewport_width_delta = (native_bounds.size.width - f64::from(viewport.width)).abs();
        let viewport_height_delta = (native_bounds.size.height - f64::from(viewport.height)).abs();
        if viewport_width_delta > 0.5 || viewport_height_delta > 0.5 {
            return Ok(None);
        }

        let frame_width_delta = (native_frame.size.width - native_bounds.size.width).abs();
        let frame_height_delta = (native_frame.size.height - native_bounds.size.height).abs();
        if frame_width_delta > 0.5 || frame_height_delta > 0.5 {
            return Err(format!(
                "the view frame must match its bounds: frame={}x{}, bounds={}x{}",
                native_frame.size.width,
                native_frame.size.height,
                native_bounds.size.width,
                native_bounds.size.height,
            ));
        }
        Ok(Some(native_frame))
    }
}

/// Converts GPUI bounds to an AppKit frame.
///
/// GPUI uses a top-left origin, and AppKit uses a bottom-left origin.
pub(super) fn appkit_frame(bounds: Bounds<Pixels>, gpui_frame: NSRect) -> NSRect {
    let x = gpui_frame.origin.x + f64::from(bounds.origin.x);
    let width = f64::from(bounds.size.width);
    let height = f64::from(bounds.size.height);
    let y = gpui_frame.origin.y + gpui_frame.size.height - f64::from(bounds.origin.y) - height;
    NSRect {
        origin: NSPoint { x, y },
        size: NSSize { width, height },
    }
}
