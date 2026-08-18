//! Tracks the native shell native views of each window.

use std::collections::HashMap;

use gpui::{App, Global, Subscription, Window, WindowId};
use objc2::MainThreadMarker;

use crate::platform::mac::chrome::NativeWindowChrome;
use crate::platform::mac::visual_effect::NativeWindowBackgroundRegistry;
use crate::window::{WindowBackground, WindowChrome};

#[derive(Default)]
struct PlatformNativeViewRegistry {
    window_backgrounds: NativeWindowBackgroundRegistry,
    chrome: HashMap<WindowId, NativeWindowChrome>,
}

#[derive(Default)]
struct NativeViewRegistry {
    native: PlatformNativeViewRegistry,
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

pub(crate) fn configure_window_chrome(
    window: &Window,
    chrome: WindowChrome,
    mtm: MainThreadMarker,
    cx: &mut App,
) -> Result<(), String> {
    crate::lifecycle::assert_installed(cx);
    let window_id = window.window_handle().window_id();
    let registry = cx.global_mut::<NativeViewRegistry>();
    registry.native.chrome.remove(&window_id);
    if let Some(native) = crate::platform::chrome::configure(window, chrome, mtm)? {
        registry.native.chrome.insert(window_id, native);
    }
    Ok(())
}

pub(crate) fn init(cx: &mut App) {
    if cx.has_global::<NativeViewRegistry>() {
        return;
    }
    let subscription = cx.on_window_closed(|cx, window_id| {
        let registry = cx.global_mut::<NativeViewRegistry>();
        registry.native.window_backgrounds.remove_window(window_id);
        registry.native.chrome.remove(&window_id);
    });
    cx.set_global(NativeViewRegistry {
        _window_closed_subscription: Some(subscription),
        ..NativeViewRegistry::default()
    });
}
