//! Configures the native window.

use gpui::Window;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSColor, NSWindowStyleMask};

use super::handle::NativeWindowHandle;

/// Applies one corner radius to the public window content view.
pub(crate) fn configure_corner_radius(
    gpui_window: &Window,
    corner_radius: f32,
    mtm: MainThreadMarker,
) -> Result<(), String> {
    let handle = NativeWindowHandle::acquire(gpui_window, mtm)?;
    let window = handle.window();
    window.setStyleMask(window.styleMask() | NSWindowStyleMask::FullSizeContentView);
    let content_view = window
        .contentView()
        .ok_or("the window content view must exist")?;
    content_view.setWantsLayer(true);
    let layer = content_view
        .layer()
        .ok_or("the window content view must have a layer")?;

    layer.setCornerRadius(f64::from(corner_radius));
    layer.setMasksToBounds(true);
    // For a non-opaque window, AppKit intersects its frame mask with drawn alpha.
    window.setOpaque(false);
    window.setBackgroundColor(Some(&NSColor::clearColor()));
    window.invalidateShadow();
    Ok(())
}
