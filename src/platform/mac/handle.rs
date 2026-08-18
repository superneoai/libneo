//! Reads the AppKit views of a GPUI window.

use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{NSView, NSWindow};
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
}
