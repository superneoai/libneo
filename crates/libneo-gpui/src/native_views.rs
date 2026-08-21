//! Tracks the native views of each window.
//!
//! GPUI elements record their bounds during paint. The registry applies the
//! bounds to the native views after the frame.

use std::collections::{HashMap, HashSet};

use gpui::{App, Global, Subscription, Window, WindowId};
use objc2::MainThreadMarker;

use crate::glass::GlassEffectFrame;
use crate::platform::mac::chrome::NativeWindowChrome;
use crate::platform::mac::glass::NativeGlassEffectRegistry;
use crate::platform::mac::table::NativeTextTableRegistry;
use crate::platform::mac::visual_effect::NativeWindowBackgroundRegistry;
use crate::table::TextTableFrame;
use crate::window::{WindowBackground, WindowChrome};

#[derive(Default)]
struct PendingFrame {
    glass_effects: HashMap<String, GlassEffectFrame>,
    text_tables: HashMap<String, TextTableFrame>,
}

#[derive(Default)]
struct PlatformNativeViewRegistry {
    glass_effects: NativeGlassEffectRegistry,
    text_tables: NativeTextTableRegistry,
    window_backgrounds: NativeWindowBackgroundRegistry,
    chrome: HashMap<WindowId, NativeWindowChrome>,
}

#[derive(Default)]
struct NativeViewRegistry {
    native: PlatformNativeViewRegistry,
    pending: HashMap<WindowId, PendingFrame>,
    scheduled: HashSet<WindowId>,
    _window_closed_subscription: Option<Subscription>,
}

impl Global for NativeViewRegistry {}

pub(crate) fn is_initialized(cx: &App) -> bool {
    cx.has_global::<NativeViewRegistry>()
}

pub(crate) fn configure_window_background(
    window: &Window,
    background: WindowBackground,
    mtm: MainThreadMarker,
    cx: &mut App,
) -> Result<(), String> {
    crate::lifecycle::assert_installed(cx);
    let window_id = window.window_handle().window_id();
    cx.global_mut::<NativeViewRegistry>()
        .native
        .window_backgrounds
        .configure(window_id, window, background, mtm)
}

pub(crate) fn configure_window_corner_radius(
    window: &Window,
    corner_radius: f32,
    mtm: MainThreadMarker,
    cx: &App,
) -> Result<(), String> {
    crate::lifecycle::assert_installed(cx);
    crate::platform::mac::window::configure_corner_radius(window, corner_radius, mtm)
}

pub(crate) fn configure_window_chrome(
    window: &Window,
    chrome: WindowChrome,
    mtm: MainThreadMarker,
    cx: &mut App,
) -> Result<(), String> {
    crate::lifecycle::assert_installed(cx);
    let window_id = window.window_handle().window_id();
    let async_cx = cx.to_async();
    let registry = cx.global_mut::<NativeViewRegistry>();
    registry.native.chrome.remove(&window_id);
    if let Some(native) = crate::platform::chrome::configure(window, chrome, mtm, async_cx)? {
        registry.native.chrome.insert(window_id, native);
    }
    Ok(())
}

pub(crate) fn record_glass_effect(
    window_id: WindowId,
    window: &Window,
    frame: GlassEffectFrame,
    mtm: MainThreadMarker,
    cx: &mut App,
) -> Result<(), String> {
    crate::lifecycle::assert_installed(cx);
    let registry = cx.global_mut::<NativeViewRegistry>();
    if !registry.pending.contains_key(&window_id) {
        return Err(crate::lifecycle::missing_root_message().to_owned());
    }
    registry
        .native
        .glass_effects
        .ensure_window(window_id, window, mtm)?;
    registry
        .pending
        .get_mut(&window_id)
        .expect("the pending frame was checked above")
        .glass_effects
        .insert(frame.id.clone(), frame);
    Ok(())
}

pub(crate) fn record_text_table(
    window_id: WindowId,
    window: &Window,
    frame: TextTableFrame,
    mtm: MainThreadMarker,
    cx: &mut App,
) -> Result<(), String> {
    crate::lifecycle::assert_installed(cx);
    let registry = cx.global_mut::<NativeViewRegistry>();
    if !registry.pending.contains_key(&window_id) {
        return Err(crate::lifecycle::missing_root_message().to_owned());
    }
    registry
        .native
        .text_tables
        .ensure_window(window_id, window, mtm)?;
    registry
        .pending
        .get_mut(&window_id)
        .expect("the pending frame was checked above")
        .text_tables
        .insert(frame.id.clone(), frame);
    Ok(())
}

/// Starts one frame, and schedules the update of the native views.
pub(crate) fn begin_frame(window: &mut Window, cx: &mut App) {
    crate::lifecycle::assert_installed(cx);
    let window_id = window.window_handle().window_id();
    let should_schedule = {
        let registry = cx.global_mut::<NativeViewRegistry>();
        registry.pending.insert(window_id, PendingFrame::default());
        registry.scheduled.insert(window_id)
    };
    if !should_schedule {
        return;
    }

    window.on_next_frame(move |window, cx| {
        let pending = {
            let registry = cx.global_mut::<NativeViewRegistry>();
            registry.scheduled.remove(&window_id);
            registry.pending.remove(&window_id).unwrap_or_default()
        };
        let mtm = MainThreadMarker::new().expect("frame callbacks run on the main thread");
        let registry = cx.global_mut::<NativeViewRegistry>();
        if let Err(error) = registry.native.text_tables.flush(
            window_id,
            window,
            pending.text_tables.into_values().collect(),
            mtm,
        ) {
            panic!("the text tables must update: {error}");
        }
        if let Err(error) = registry.native.glass_effects.flush(
            window_id,
            window,
            pending.glass_effects.into_values().collect(),
            mtm,
        ) {
            panic!("the glass effects must update: {error}");
        }
    });
}

pub(crate) fn init(cx: &mut App) {
    if cx.has_global::<NativeViewRegistry>() {
        return;
    }
    let subscription = cx.on_window_closed(|cx, window_id| {
        let registry = cx.global_mut::<NativeViewRegistry>();
        registry.pending.remove(&window_id);
        registry.scheduled.remove(&window_id);
        registry.native.glass_effects.remove_window(window_id);
        registry.native.text_tables.remove_window(window_id);
        registry.native.window_backgrounds.remove_window(window_id);
        registry.native.chrome.remove(&window_id);
    });
    cx.set_global(NativeViewRegistry {
        _window_closed_subscription: Some(subscription),
        ..NativeViewRegistry::default()
    });
}
