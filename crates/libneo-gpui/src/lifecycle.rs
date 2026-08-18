//! Integrates libneo-gpui with an existing GPUI application and its windows.

use gpui::colors::GlobalColors;
use gpui::{App, Context, Entity, Global, IntoElement, Render, Window};
use objc2::MainThreadMarker;

const INSTALL_MESSAGE: &str =
    "libneo-gpui is not installed; call libneo::install(cx) before opening windows";
const ROOT_MESSAGE: &str =
    "libneo-gpui native elements require each GPUI window root to be wrapped in libneo::NativeRoot";

struct Lifecycle;

impl Global for Lifecycle {}

/// Initializes libneo-gpui for an existing GPUI application.
///
/// Call this once in the callback passed to `gpui_platform::Application::run`,
/// before opening any windows. Calling it again on the same [`App`] is safe and
/// leaves existing colors, theme state, and native registries intact.
///
/// Every window that uses libneo-gpui native elements must also wrap its root
/// entity in [`NativeRoot`]. [`crate::window::run`] performs both steps
/// automatically.
///
/// # Panics
///
/// This function panics when called off the main thread or on a system before
/// macOS 26.1.
pub fn install(cx: &mut App) {
    crate::platform::assert_supported_os();
    install_components(cx);
}

fn install_components(cx: &mut App) {
    let plan = InstallPlan::for_state(ComponentState {
        colors: cx.has_global::<GlobalColors>(),
        theme: cx.has_global::<crate::theme::Theme>(),
        native_views: crate::native_views::is_initialized(cx),
        lifecycle: cx.has_global::<Lifecycle>(),
    });
    if plan.colors {
        cx.init_colors();
    }
    if plan.theme {
        crate::theme::init(cx);
    }
    if plan.native_views {
        crate::native_views::init(cx);
    }
    if plan.lifecycle {
        cx.set_global(Lifecycle);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ComponentState {
    colors: bool,
    theme: bool,
    native_views: bool,
    lifecycle: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct InstallPlan {
    colors: bool,
    theme: bool,
    native_views: bool,
    lifecycle: bool,
}

impl InstallPlan {
    fn for_state(state: ComponentState) -> Self {
        Self {
            colors: !state.colors,
            theme: !state.theme,
            native_views: !state.native_views,
            lifecycle: !state.lifecycle,
        }
    }
}

pub(crate) fn assert_installed(cx: &App) {
    assert!(cx.has_global::<Lifecycle>(), "{INSTALL_MESSAGE}");
}

pub(crate) fn main_thread_marker(cx: &App) -> MainThreadMarker {
    assert_installed(cx);
    MainThreadMarker::new().expect("libneo-gpui lifecycle operations must run on the main thread")
}

pub(crate) fn missing_root_message() -> &'static str {
    ROOT_MESSAGE
}

/// Wraps a GPUI window root and reconciles its native views each frame.
///
/// Create the application's existing root entity as usual, pass it to
/// [`NativeRoot::new`], and return an entity containing this wrapper from
/// `App::open_window`. Use one wrapper for every window that renders libneo-gpui
/// glass effect or native text table elements.
///
/// Call [`install`] before opening the window. Rendering this wrapper without
/// installation panics with an order-specific message.
pub struct NativeRoot<V: Render> {
    content: Entity<V>,
}

impl<V: Render> NativeRoot<V> {
    /// Creates a native lifecycle root around an existing GPUI root entity.
    ///
    /// [`install`] must have been called on the application before the wrapper
    /// is first rendered.
    pub fn new(content: Entity<V>) -> Self {
        Self { content }
    }
}

impl<V: Render> Render for NativeRoot<V> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        assert_installed(cx);
        crate::theme::observe_window(window, cx);
        crate::native_views::begin_frame(window, cx);
        self.content.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::{ComponentState, INSTALL_MESSAGE, InstallPlan};

    #[test]
    fn missing_install_diagnostic_is_actionable() {
        assert_eq!(
            INSTALL_MESSAGE,
            "libneo-gpui is not installed; call libneo::install(cx) before opening windows"
        );
    }

    #[test]
    fn first_install_initializes_every_component() {
        let plan = InstallPlan::for_state(ComponentState {
            colors: false,
            theme: false,
            native_views: false,
            lifecycle: false,
        });

        assert_eq!(
            plan,
            InstallPlan {
                colors: true,
                theme: true,
                native_views: true,
                lifecycle: true,
            }
        );
    }

    #[test]
    fn repeated_install_preserves_every_component() {
        let plan = InstallPlan::for_state(ComponentState {
            colors: true,
            theme: true,
            native_views: true,
            lifecycle: true,
        });

        assert_eq!(
            plan,
            InstallPlan {
                colors: false,
                theme: false,
                native_views: false,
                lifecycle: false,
            }
        );
    }

    #[test]
    fn partial_install_only_initializes_missing_components() {
        let plan = InstallPlan::for_state(ComponentState {
            colors: true,
            theme: false,
            native_views: true,
            lifecycle: false,
        });

        assert_eq!(
            plan,
            InstallPlan {
                colors: false,
                theme: true,
                native_views: false,
                lifecycle: true,
            }
        );
    }
}
