//! Builds the native glass effects.

use std::collections::{HashMap, HashSet};

use gpui::{Rgba, Window, WindowId};
use objc2::rc::Retained;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSAutoresizingMaskOptions, NSColor, NSGlassEffectContainerView, NSGlassEffectView,
    NSGlassEffectViewStyle, NSTextField, NSView, NSWindowOrderingMode,
};
use objc2_foundation::{NSPoint, NSRect, NSString};

use crate::glass::{
    GlassEffectConfiguration, GlassEffectContent, GlassEffectFrame, GlassEffectStyle,
};

use super::handle::{NativeWindowHandle, appkit_frame};

/// Owns the glass effects of each window.
#[derive(Default)]
pub(crate) struct NativeGlassEffectRegistry {
    windows: HashMap<WindowId, NativeGlassEffectWindow>,
}

impl NativeGlassEffectRegistry {
    /// Releases the effects of a closed window.
    pub(crate) fn remove_window(&mut self, window_id: WindowId) {
        self.windows.remove(&window_id);
    }

    /// Prepares the window that holds the effects.
    pub(crate) fn ensure_window(
        &mut self,
        window_id: WindowId,
        gpui_window: &Window,
        mtm: MainThreadMarker,
    ) -> Result<(), String> {
        if let std::collections::hash_map::Entry::Vacant(entry) = self.windows.entry(window_id) {
            entry.insert(NativeGlassEffectWindow::new(gpui_window, mtm)?);
        }
        Ok(())
    }

    /// Applies the recorded bounds to the native views.
    pub(crate) fn flush(
        &mut self,
        window_id: WindowId,
        gpui_window: &mut Window,
        frames: Vec<GlassEffectFrame>,
        mtm: MainThreadMarker,
    ) -> Result<(), String> {
        let Some(native) = self.windows.get_mut(&window_id) else {
            return Ok(());
        };
        native.flush(gpui_window, frames, mtm)
    }
}

/// Holds the glass effects of one window.
struct NativeGlassEffectWindow {
    handle: NativeWindowHandle,
    groups: HashMap<String, NativeGlassEffectGroup>,
    ungrouped_effects: HashMap<String, NativeGlassEffect>,
    locations: HashMap<String, Option<String>>,
}

impl NativeGlassEffectWindow {
    fn new(gpui_window: &Window, mtm: MainThreadMarker) -> Result<Self, String> {
        Ok(Self {
            handle: NativeWindowHandle::acquire(gpui_window, mtm)?,
            groups: HashMap::new(),
            ungrouped_effects: HashMap::new(),
            locations: HashMap::new(),
        })
    }

    fn flush(
        &mut self,
        gpui_window: &mut Window,
        frames: Vec<GlassEffectFrame>,
        mtm: MainThreadMarker,
    ) -> Result<(), String> {
        let Some(gpui_frame) = self.handle.gpui_frame_in_superview(gpui_window)? else {
            // Wait until GPUI and AppKit report the same view size.
            gpui_window.refresh();
            return Ok(());
        };
        let superview_bounds = self.handle.superview().bounds();
        let live_effects: HashSet<_> = frames.iter().map(|effect| effect.id.clone()).collect();

        for group in self.groups.values() {
            group.sync_container_frame(superview_bounds);
        }

        for effect in frames {
            let effect_id = effect.id.clone();
            let target_group = effect
                .configuration
                .group
                .as_ref()
                .map(|group| group.id.clone());
            self.remove_if_parent_changed(&effect.id, &target_group);

            if let Some(group_configuration) = &effect.configuration.group {
                let group = match self.groups.entry(group_configuration.id.clone()) {
                    std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        entry.insert(NativeGlassEffectGroup::new(
                            &self.handle,
                            superview_bounds,
                            f64::from(group_configuration.spacing),
                            mtm,
                        )?)
                    }
                };
                group.set_spacing(f64::from(group_configuration.spacing));
                group.sync_container_frame(superview_bounds);
                let frame = appkit_frame(effect.bounds, gpui_frame);
                group.upsert_effect(effect, frame, mtm);
            } else {
                let frame = appkit_frame(effect.bounds, gpui_frame);
                upsert_ungrouped_effect(
                    &mut self.ungrouped_effects,
                    &self.handle,
                    effect,
                    frame,
                    mtm,
                );
            }
            self.locations.insert(effect_id, target_group);
        }
        self.retain_live_effects(&live_effects);
        Ok(())
    }

    fn retain_live_effects(&mut self, live_effects: &HashSet<String>) {
        retain_live(&mut self.ungrouped_effects, live_effects);
        for group in self.groups.values_mut() {
            retain_live(&mut group.effects, live_effects);
        }
        self.groups.retain(|_, group| !group.effects.is_empty());
        retain_live(&mut self.locations, live_effects);
    }

    fn remove_if_parent_changed(&mut self, id: &str, target: &Option<String>) {
        let Some(previous) = self.locations.get(id) else {
            return;
        };
        if previous == target {
            return;
        }
        match previous {
            Some(group) => {
                if let Some(group) = self.groups.get_mut(group) {
                    group.effects.remove(id);
                }
            }
            None => {
                self.ungrouped_effects.remove(id);
            }
        }
    }
}

fn retain_live<T>(entries: &mut HashMap<String, T>, live_effects: &HashSet<String>) {
    entries.retain(|id, _| live_effects.contains(id));
}

/// Holds one `NSGlassEffectContainerView` and its effects.
struct NativeGlassEffectGroup {
    container: Retained<NSGlassEffectContainerView>,
    host: Retained<NSView>,
    effects: HashMap<String, NativeGlassEffect>,
}

impl NativeGlassEffectGroup {
    fn new(
        window: &NativeWindowHandle,
        bounds: NSRect,
        spacing: f64,
        mtm: MainThreadMarker,
    ) -> Result<Self, String> {
        let container = NSGlassEffectContainerView::initWithFrame(
            NSGlassEffectContainerView::alloc(mtm),
            bounds,
        );
        let host = NSView::initWithFrame(NSView::alloc(mtm), bounds);
        let flexible = NSAutoresizingMaskOptions::ViewWidthSizable
            | NSAutoresizingMaskOptions::ViewHeightSizable;
        container.setAutoresizingMask(flexible);
        host.setAutoresizingMask(flexible);
        container.setSpacing(spacing);
        container.setContentView(Some(&host));
        debug_assert!((container.spacing() - spacing).abs() <= f64::EPSILON);
        debug_assert!(container.contentView().is_some());
        window.superview().addSubview_positioned_relativeTo(
            &container,
            NSWindowOrderingMode::Above,
            Some(window.gpui_view()),
        );
        Ok(Self {
            container,
            host,
            effects: HashMap::new(),
        })
    }

    fn set_spacing(&self, spacing: f64) {
        self.container.setSpacing(spacing);
    }

    fn sync_container_frame(&self, bounds: NSRect) {
        self.container.setFrame(bounds);
        self.host.setFrame(bounds);
    }

    fn upsert_effect(&mut self, effect: GlassEffectFrame, frame: NSRect, mtm: MainThreadMarker) {
        match self.effects.entry(effect.id) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                if entry.get().configuration != effect.configuration {
                    entry.insert(NativeGlassEffect::new(
                        &self.host,
                        None,
                        effect.configuration,
                        frame,
                        mtm,
                    ));
                } else {
                    entry.get().view.setFrame(frame);
                }
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(NativeGlassEffect::new(
                    &self.host,
                    None,
                    effect.configuration,
                    frame,
                    mtm,
                ));
            }
        }
    }
}

impl Drop for NativeGlassEffectGroup {
    fn drop(&mut self) {
        debug_assert!(
            MainThreadMarker::new().is_some(),
            "the glass effect group releases on the main thread"
        );
        self.container.removeFromSuperview();
    }
}

fn upsert_ungrouped_effect(
    effects: &mut HashMap<String, NativeGlassEffect>,
    window: &NativeWindowHandle,
    effect: GlassEffectFrame,
    frame: NSRect,
    mtm: MainThreadMarker,
) {
    match effects.entry(effect.id) {
        std::collections::hash_map::Entry::Occupied(mut entry) => {
            if entry.get().configuration != effect.configuration {
                entry.insert(NativeGlassEffect::new(
                    window.superview(),
                    Some(window.gpui_view()),
                    effect.configuration,
                    frame,
                    mtm,
                ));
            } else {
                entry.get().view.setFrame(frame);
            }
        }
        std::collections::hash_map::Entry::Vacant(entry) => {
            entry.insert(NativeGlassEffect::new(
                window.superview(),
                Some(window.gpui_view()),
                effect.configuration,
                frame,
                mtm,
            ));
        }
    }
}

/// Holds one `NSGlassEffectView` and its native content.
struct NativeGlassEffect {
    view: Retained<NSGlassEffectView>,
    _content: Option<Retained<NSView>>,
    configuration: GlassEffectConfiguration,
}

impl NativeGlassEffect {
    fn new(
        parent: &NSView,
        relative_to: Option<&NSView>,
        configuration: GlassEffectConfiguration,
        frame: NSRect,
        mtm: MainThreadMarker,
    ) -> Self {
        let view = NSGlassEffectView::initWithFrame(NSGlassEffectView::alloc(mtm), frame);
        view.setStyle(match configuration.style {
            GlassEffectStyle::Regular => NSGlassEffectViewStyle::Regular,
            GlassEffectStyle::Clear => NSGlassEffectViewStyle::Clear,
        });
        view.setCornerRadius(f64::from(configuration.corner_radius));
        let tint = configuration.tint.map(ns_color);
        view.setTintColor(tint.as_deref());

        let content = configuration.content.as_ref().map(|content| match content {
            GlassEffectContent::Label(text) => {
                let text = NSString::from_str(text);
                let label = NSTextField::labelWithString(&text, mtm);
                label.setFrame(local_bounds(frame));
                label.setAutoresizingMask(
                    NSAutoresizingMaskOptions::ViewWidthSizable
                        | NSAutoresizingMaskOptions::ViewHeightSizable,
                );
                label.into_super().into_super()
            }
        });
        view.setContentView(content.as_deref());

        if let Some(relative_to) = relative_to {
            parent.addSubview_positioned_relativeTo(
                &view,
                NSWindowOrderingMode::Above,
                Some(relative_to),
            );
        } else {
            parent.addSubview(&view);
        }

        Self {
            view,
            _content: content,
            configuration,
        }
    }
}

impl Drop for NativeGlassEffect {
    fn drop(&mut self) {
        debug_assert!(
            MainThreadMarker::new().is_some(),
            "the glass effect releases on the main thread"
        );
        self.view.removeFromSuperview();
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

fn local_bounds(frame: NSRect) -> NSRect {
    NSRect {
        origin: NSPoint { x: 0.0, y: 0.0 },
        size: frame.size,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use gpui::{Bounds, point, px, size};
    use objc2_foundation::{NSPoint, NSRect, NSSize};

    use super::retain_live;
    use crate::platform::mac::handle::appkit_frame;

    fn assert_rect(actual: NSRect, expected: NSRect) {
        assert!((actual.origin.x - expected.origin.x).abs() < f64::EPSILON);
        assert!((actual.origin.y - expected.origin.y).abs() < f64::EPSILON);
        assert!((actual.size.width - expected.size.width).abs() < f64::EPSILON);
        assert!((actual.size.height - expected.size.height).abs() < f64::EPSILON);
    }

    #[test]
    fn maps_full_size_transparent_title_bar_coordinates() {
        let bounds: Bounds<_> = Bounds::new(point(px(24.0), px(104.0)), size(px(224.0), px(430.0)));
        let gpui_frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1120.0, 700.0));

        assert_rect(
            appkit_frame(bounds, gpui_frame),
            NSRect::new(NSPoint::new(24.0, 166.0), NSSize::new(224.0, 430.0)),
        );
    }

    #[test]
    fn maps_inset_opaque_title_bar_coordinates() {
        let bounds: Bounds<_> = Bounds::new(point(px(24.0), px(104.0)), size(px(224.0), px(430.0)));
        let gpui_frame = NSRect::new(NSPoint::new(8.0, 32.0), NSSize::new(1120.0, 668.0));

        assert_rect(
            appkit_frame(bounds, gpui_frame),
            NSRect::new(NSPoint::new(32.0, 166.0), NSSize::new(224.0, 430.0)),
        );
    }

    #[test]
    fn reconciliation_drops_effects_absent_from_the_current_frame() {
        let mut entries = HashMap::from([("removed".to_owned(), 1), ("retained".to_owned(), 2)]);
        let live_effects = HashSet::from(["retained".to_owned()]);

        retain_live(&mut entries, &live_effects);

        assert_eq!(entries, HashMap::from([("retained".to_owned(), 2)]));
    }
}
