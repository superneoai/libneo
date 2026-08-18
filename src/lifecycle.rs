//! Integrates libneo with an existing GPUI application and its windows.

use gpui::colors::GlobalColors;
use gpui::{App, Context, Entity, Global, IntoElement, Render, Window};

const INSTALL_MESSAGE: &str =
    "libneo is not installed; call libneo::install(cx) before opening windows";

struct Lifecycle;

impl Global for Lifecycle {}

/// Initializes libneo for an existing GPUI application.
///
/// Call this once before opening any windows. Calling it again on the same
/// [`App`] preserves existing colors and theme state.
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
        lifecycle: cx.has_global::<Lifecycle>(),
    });
    if plan.colors {
        cx.init_colors();
    }
    if plan.theme {
        crate::theme::init(cx);
    }
    if plan.lifecycle {
        cx.set_global(Lifecycle);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ComponentState {
    colors: bool,
    theme: bool,
    lifecycle: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct InstallPlan {
    colors: bool,
    theme: bool,
    lifecycle: bool,
}

impl InstallPlan {
    fn for_state(state: ComponentState) -> Self {
        Self {
            colors: !state.colors,
            theme: !state.theme,
            lifecycle: !state.lifecycle,
        }
    }
}

pub(crate) fn assert_installed(cx: &App) {
    assert!(cx.has_global::<Lifecycle>(), "{INSTALL_MESSAGE}");
}

/// Wraps a GPUI window root and observes its native appearance each frame.
pub struct NativeRoot<V: Render> {
    content: Entity<V>,
}

impl<V: Render> NativeRoot<V> {
    /// Creates a lifecycle root around an existing GPUI root entity.
    pub fn new(content: Entity<V>) -> Self {
        Self { content }
    }
}

impl<V: Render> Render for NativeRoot<V> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        assert_installed(cx);
        crate::theme::observe_window(window, cx);
        self.content.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::{ComponentState, InstallPlan};

    #[test]
    fn first_install_initializes_every_component() {
        let plan = InstallPlan::for_state(ComponentState {
            colors: false,
            theme: false,
            lifecycle: false,
        });

        assert_eq!(
            plan,
            InstallPlan {
                colors: true,
                theme: true,
                lifecycle: true,
            }
        );
    }

    #[test]
    fn repeated_install_preserves_every_component() {
        let plan = InstallPlan::for_state(ComponentState {
            colors: true,
            theme: true,
            lifecycle: true,
        });

        assert_eq!(
            plan,
            InstallPlan {
                colors: false,
                theme: false,
                lifecycle: false,
            }
        );
    }

    #[test]
    fn partial_install_only_initializes_missing_components() {
        let plan = InstallPlan::for_state(ComponentState {
            colors: true,
            theme: false,
            lifecycle: false,
        });

        assert_eq!(
            plan,
            InstallPlan {
                colors: false,
                theme: true,
                lifecycle: true,
            }
        );
    }
}
