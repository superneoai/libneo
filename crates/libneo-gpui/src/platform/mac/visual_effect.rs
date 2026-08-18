//! Builds the window visual effect backgrounds.

use std::collections::HashMap;

use gpui::{Window, WindowId};
use objc2::rc::Retained;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSAutoresizingMaskOptions, NSVisualEffectBlendingMode, NSVisualEffectMaterial,
    NSVisualEffectState, NSVisualEffectView, NSWindowOrderingMode,
};

use crate::window::{VisualEffectMaterial, WindowBackground};

use super::handle::NativeWindowHandle;

/// Owns the visual effect background of each window.
#[derive(Default)]
pub(crate) struct NativeWindowBackgroundRegistry {
    windows: HashMap<WindowId, NativeVisualEffectBackground>,
}

impl NativeWindowBackgroundRegistry {
    /// Applies the background to a window.
    pub(crate) fn configure(
        &mut self,
        window_id: WindowId,
        gpui_window: &Window,
        background: WindowBackground,
        mtm: MainThreadMarker,
    ) -> Result<(), String> {
        self.windows.remove(&window_id);
        if let WindowBackground::VisualEffect(visual_effect_material) = background {
            self.windows.insert(
                window_id,
                NativeVisualEffectBackground::new(gpui_window, visual_effect_material, mtm)?,
            );
        }
        Ok(())
    }

    /// Releases the background of a closed window.
    pub(crate) fn remove_window(&mut self, window_id: WindowId) {
        self.windows.remove(&window_id);
    }
}

/// Holds the visual effect view of one window.
struct NativeVisualEffectBackground {
    view: Retained<NSVisualEffectView>,
}

impl NativeVisualEffectBackground {
    /// Adds the visual effect view below the GPUI view.
    fn new(
        gpui_window: &Window,
        visual_effect_material: VisualEffectMaterial,
        mtm: MainThreadMarker,
    ) -> Result<Self, String> {
        let handle = NativeWindowHandle::acquire(gpui_window, mtm)?;
        let view = NSVisualEffectView::initWithFrame(
            NSVisualEffectView::alloc(mtm),
            handle.gpui_view().frame(),
        );
        view.setMaterial(match visual_effect_material {
            VisualEffectMaterial::UnderWindowBackground => {
                NSVisualEffectMaterial::UnderWindowBackground
            }
            VisualEffectMaterial::HudWindow => NSVisualEffectMaterial::HUDWindow,
            VisualEffectMaterial::Sidebar => NSVisualEffectMaterial::Sidebar,
        });
        view.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
        view.setState(NSVisualEffectState::Active);
        view.setAutoresizingMask(
            NSAutoresizingMaskOptions::ViewWidthSizable
                | NSAutoresizingMaskOptions::ViewHeightSizable,
        );
        handle.superview().addSubview_positioned_relativeTo(
            &view,
            NSWindowOrderingMode::Below,
            Some(handle.gpui_view()),
        );
        Ok(Self { view })
    }
}

impl Drop for NativeVisualEffectBackground {
    fn drop(&mut self) {
        debug_assert!(
            MainThreadMarker::new().is_some(),
            "the background releases on the main thread"
        );
        self.view.removeFromSuperview();
    }
}
