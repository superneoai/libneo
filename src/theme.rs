//! Supplies the colors of the application.
//!
//! [`Theme`] holds the active palette. The application reads the colors with
//! [`Theme::global`].

use std::collections::HashMap;

use gpui::{App, Global, Refineable, Rgba, Subscription, Window, WindowAppearance, WindowId, rgba};

/// Selects the source of the appearance.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ThemeMode {
    /// Uses the system appearance.
    #[default]
    FollowSystem,
    /// Uses the light appearance.
    Light,
    /// Uses the dark appearance.
    Dark,
}

/// Shows the active appearance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThemeAppearance {
    /// Shows the light appearance.
    Light,
    /// Shows the dark appearance.
    Dark,
}

impl From<WindowAppearance> for ThemeAppearance {
    fn from(appearance: WindowAppearance) -> Self {
        match appearance {
            WindowAppearance::Light | WindowAppearance::VibrantLight => Self::Light,
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Self::Dark,
        }
    }
}

/// Holds the colors of one palette.
///
/// [`ThemeTokensRefinement`] changes selected colors.
#[derive(Clone, Debug, PartialEq, Refineable)]
#[refineable(Debug, PartialEq)]
pub struct ThemeTokens {
    /// Fills the window background.
    pub background: Rgba,
    /// Fills the background of grouped content.
    pub grouped_background: Rgba,
    /// Colors the primary text.
    pub text: Rgba,
    /// Colors the secondary text.
    pub muted_text: Rgba,
    /// Colors the interactive elements.
    pub accent: Rgba,
    /// Tints the glass effects.
    pub glass_tint: Rgba,
}

impl ThemeTokens {
    /// Returns the light palette.
    pub fn light() -> Self {
        Self {
            background: rgba(0xf4f7fbff),
            grouped_background: rgba(0xffffffff),
            text: rgba(0x172033ff),
            muted_text: rgba(0x5d687cff),
            accent: rgba(0x2868d8ff),
            glass_tint: rgba(0x4a8cf044),
        }
    }

    /// Returns the dark palette.
    pub fn dark() -> Self {
        Self {
            background: rgba(0x101522ff),
            grouped_background: rgba(0x1b2333ff),
            text: rgba(0xf4f7ffff),
            muted_text: rgba(0xa9b4c7ff),
            accent: rgba(0x79aaffff),
            glass_tint: rgba(0x5b8dff55),
        }
    }
}

/// Holds the active palette and the appearance.
pub struct Theme {
    mode: ThemeMode,
    appearance: ThemeAppearance,
    tokens: ThemeTokens,
    light: ThemeTokens,
    dark: ThemeTokens,
    appearance_subscriptions: HashMap<WindowId, Subscription>,
    _window_closed_subscription: Option<Subscription>,
}

impl Global for Theme {}

impl Theme {
    /// Returns the active theme.
    ///
    /// # Panics
    ///
    /// This function panics if [`crate::install`] has not initialized libneo.
    pub fn global(cx: &App) -> &Self {
        crate::lifecycle::assert_installed(cx);
        cx.global::<Self>()
    }

    /// Returns the theme mode.
    pub fn mode(&self) -> ThemeMode {
        self.mode
    }

    /// Returns the active appearance.
    pub fn appearance(&self) -> ThemeAppearance {
        self.appearance
    }

    /// Returns the active colors.
    pub fn tokens(&self) -> &ThemeTokens {
        &self.tokens
    }

    /// Sets the theme mode, and applies the appearance to the windows.
    ///
    /// # Panics
    ///
    /// This function panics if [`crate::install`] has not initialized libneo.
    pub fn set_mode(mode: ThemeMode, cx: &mut App) {
        crate::lifecycle::assert_installed(cx);
        let window_override = match mode {
            ThemeMode::FollowSystem => None,
            ThemeMode::Light => Some(WindowAppearance::Light),
            ThemeMode::Dark => Some(WindowAppearance::Dark),
        };
        cx.set_window_appearance(window_override);

        let appearance = match mode {
            ThemeMode::FollowSystem => cx.window_appearance().into(),
            ThemeMode::Light => ThemeAppearance::Light,
            ThemeMode::Dark => ThemeAppearance::Dark,
        };
        cx.global_mut::<Self>().select(mode, appearance);
        cx.refresh_windows();
    }

    fn select(&mut self, mode: ThemeMode, appearance: ThemeAppearance) {
        self.mode = mode;
        self.appearance = appearance;
        self.tokens = match appearance {
            ThemeAppearance::Light => self.light.clone(),
            ThemeAppearance::Dark => self.dark.clone(),
        };
    }
}

pub(crate) fn init(cx: &mut App) {
    if cx.has_global::<Theme>() {
        return;
    }

    let appearance = ThemeAppearance::from(cx.window_appearance());
    let light = ThemeTokens::light();
    let dark = ThemeTokens::dark();
    let tokens = match appearance {
        ThemeAppearance::Light => light.clone(),
        ThemeAppearance::Dark => dark.clone(),
    };
    let window_closed_subscription = cx.on_window_closed(|cx, window_id| {
        if cx.has_global::<Theme>() {
            cx.global_mut::<Theme>()
                .appearance_subscriptions
                .remove(&window_id);
        }
    });

    cx.set_global(Theme {
        mode: ThemeMode::FollowSystem,
        appearance,
        tokens,
        light,
        dark,
        appearance_subscriptions: HashMap::new(),
        _window_closed_subscription: Some(window_closed_subscription),
    });
}

pub(crate) fn observe_window(window: &mut Window, cx: &mut App) {
    let window_id = window.window_handle().window_id();
    if cx
        .global::<Theme>()
        .appearance_subscriptions
        .contains_key(&window_id)
    {
        return;
    }

    let subscription = window.observe_window_appearance(|window, cx| {
        let appearance = ThemeAppearance::from(window.appearance());
        let theme = cx.global_mut::<Theme>();
        if theme.mode == ThemeMode::FollowSystem && theme.appearance != appearance {
            theme.select(ThemeMode::FollowSystem, appearance);
            window.refresh();
        }
    });
    cx.global_mut::<Theme>()
        .appearance_subscriptions
        .insert(window_id, subscription);
}

#[cfg(test)]
mod tests {
    use gpui::{Refineable, rgba};

    use super::{ThemeAppearance, ThemeTokens, ThemeTokensRefinement};

    #[test]
    fn refinement_changes_selected_colors() {
        let base = ThemeTokens::light();
        let accent = rgba(0xff3366ff);
        let refined = base.clone().refined(ThemeTokensRefinement {
            accent: Some(accent),
            ..ThemeTokensRefinement::default()
        });

        assert_eq!(refined.accent, accent);
        assert_eq!(refined.background, base.background);
        assert_eq!(refined.text, base.text);
    }

    #[test]
    fn vibrant_appearance_maps_to_base_appearance() {
        assert_eq!(
            ThemeAppearance::from(gpui::WindowAppearance::VibrantLight),
            ThemeAppearance::Light
        );
        assert_eq!(
            ThemeAppearance::from(gpui::WindowAppearance::VibrantDark),
            ThemeAppearance::Dark
        );
    }
}
