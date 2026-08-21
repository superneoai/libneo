//! Builds the window visual effect backgrounds.

use std::collections::HashMap;

use block2::RcBlock;
use gpui::{Window, WindowId};
use objc2::rc::Retained;
use objc2::runtime::Bool;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSAutoresizingMaskOptions, NSBezierPath, NSColor, NSImage, NSImageResizingMode,
    NSViewController, NSVisualEffectBlendingMode, NSVisualEffectMaterial, NSVisualEffectState,
    NSVisualEffectView,
};
use objc2_foundation::{NSEdgeInsets, NSSize};

use crate::window::{VisualEffectMaterial, WindowBackground};

use super::handle::NativeWindowHandle;

/// Owns the visual effect background of each window.
#[derive(Default)]
pub(crate) struct NativeWindowBackgroundRegistry {
    windows: HashMap<WindowId, NativeVisualEffectBackground>,
}

impl NativeWindowBackgroundRegistry {
    /// Applies the background and corner shape to a window.
    pub(crate) fn configure(
        &mut self,
        window_id: WindowId,
        gpui_window: &Window,
        background: WindowBackground,
        corner_radius: Option<f32>,
        mtm: MainThreadMarker,
    ) -> Result<(), String> {
        self.windows.remove(&window_id);
        if matches!(background, WindowBackground::VisualEffect(_)) || corner_radius.is_some() {
            self.windows.insert(
                window_id,
                NativeVisualEffectBackground::new(gpui_window, background, corner_radius, mtm)?,
            );
        }
        Ok(())
    }

    /// Releases the background of a closed window.
    pub(crate) fn remove_window(&mut self, window_id: WindowId) {
        self.windows.remove(&window_id);
    }
}

/// Holds the visual effect content root of one window.
struct NativeVisualEffectBackground {
    _view: Retained<NSVisualEffectView>,
    _root_controller: Option<Retained<NSViewController>>,
    _mask_image: Option<Retained<NSImage>>,
}

impl NativeVisualEffectBackground {
    /// Puts the GPUI host inside a visual effect window content root.
    fn new(
        gpui_window: &Window,
        background: WindowBackground,
        corner_radius: Option<f32>,
        mtm: MainThreadMarker,
    ) -> Result<Self, String> {
        let handle = NativeWindowHandle::acquire(gpui_window, mtm)?;
        let window = handle.window();
        let content = window
            .contentView()
            .ok_or("the window content view must exist")?;
        let view =
            NSVisualEffectView::initWithFrame(NSVisualEffectView::alloc(mtm), content.frame());
        match background {
            WindowBackground::Standard => {
                view.setBlendingMode(NSVisualEffectBlendingMode::WithinWindow);
            }
            WindowBackground::VisualEffect(material) => {
                view.setMaterial(appkit_material(material));
                view.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
                view.setState(NSVisualEffectState::Active);
            }
        }
        view.setAutoresizingMask(
            NSAutoresizingMaskOptions::ViewWidthSizable
                | NSAutoresizingMaskOptions::ViewHeightSizable,
        );

        let mask_image = corner_radius.map(|radius| rounded_mask_image(f64::from(radius)));
        if let (Some(radius), Some(mask_image)) = (corner_radius, mask_image.as_deref()) {
            view.setMaskImage(Some(mask_image));
            view.setWantsLayer(true);
            let layer = view
                .layer()
                .ok_or("the visual effect content root must have a layer")?;
            layer.setCornerRadius(f64::from(radius));
            // NSVisualEffectView.maskImage does not clip subviews.
            layer.setMasksToBounds(true);
        }

        content.setFrame(view.bounds());
        content.setAutoresizingMask(
            NSAutoresizingMaskOptions::ViewWidthSizable
                | NSAutoresizingMaskOptions::ViewHeightSizable,
        );
        view.addSubview(&content);

        let root_controller = window.contentViewController().map(|content_controller| {
            let root_controller = NSViewController::new(mtm);
            root_controller.setView(&view);
            root_controller.addChildViewController(&content_controller);
            window.setContentViewController(Some(&root_controller));
            root_controller
        });
        if root_controller.is_none() {
            window.setContentView(Some(&view));
        }

        window.setOpaque(false);
        // GPUI's 0.0001-alpha transparent background leaves the system-radius hairline visible.
        window.setBackgroundColor(Some(&NSColor::clearColor()));
        window.invalidateShadow();

        Ok(Self {
            _view: view,
            _root_controller: root_controller,
            _mask_image: mask_image,
        })
    }
}

fn appkit_material(material: VisualEffectMaterial) -> NSVisualEffectMaterial {
    match material {
        VisualEffectMaterial::UnderWindowBackground => {
            NSVisualEffectMaterial::UnderWindowBackground
        }
        VisualEffectMaterial::HudWindow => NSVisualEffectMaterial::HUDWindow,
        VisualEffectMaterial::Sidebar => NSVisualEffectMaterial::Sidebar,
    }
}

fn rounded_mask_image(radius: f64) -> Retained<NSImage> {
    let side = radius.mul_add(2.0, 1.0);
    let drawing_handler = RcBlock::new(move |rect| {
        NSColor::whiteColor().setFill();
        NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(rect, radius, radius).fill();
        Bool::YES
    });
    let image = NSImage::imageWithSize_flipped_drawingHandler(
        NSSize::new(side, side),
        false,
        &drawing_handler,
    );
    image.setCapInsets(NSEdgeInsets {
        top: radius,
        left: radius,
        bottom: radius,
        right: radius,
    });
    image.setResizingMode(NSImageResizingMode(0));
    image
}
