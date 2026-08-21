//! Resolves AppKit system colors for GPUI drawing.
//!
//! GPUI draws plain pixels and does not participate in AppKit vibrancy. Colors
//! are therefore resolved against Aqua or Dark Aqua, including their Increase
//! Contrast counterparts, but never the vibrant appearances: vibrant colors
//! are pre-compensated for a blend GPUI does not apply, which unnecessarily
//! reduces contrast when drawn directly.

use gpui::{App, Rgba, Subscription, Window, WindowAppearance};

/// Selects the non-vibrant AppKit appearance used to resolve a system color.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Appearance {
    /// Resolves against Aqua and its Increase Contrast counterpart.
    Aqua,
    /// Resolves against Dark Aqua and its Increase Contrast counterpart.
    DarkAqua,
}

impl From<WindowAppearance> for Appearance {
    fn from(appearance: WindowAppearance) -> Self {
        match appearance {
            WindowAppearance::Light | WindowAppearance::VibrantLight => Self::Aqua,
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Self::DarkAqua,
        }
    }
}

/// Selects an AppKit system color to resolve.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemColor {
    /// Maps to AppKit's `labelColor`.
    LabelColor,
    /// Maps to AppKit's `secondaryLabelColor`.
    SecondaryLabelColor,
    /// Maps to AppKit's `tertiaryLabelColor`.
    TertiaryLabelColor,
    /// Maps to AppKit's `quaternaryLabelColor`.
    QuaternaryLabelColor,
    /// Maps to AppKit's `quinaryLabelColor`.
    QuinaryLabelColor,
    /// Maps to AppKit's `placeholderTextColor`.
    PlaceholderTextColor,
    /// Maps to AppKit's `linkColor`.
    LinkColor,
    /// Maps to AppKit's `separatorColor`.
    SeparatorColor,
    /// Maps to AppKit's `gridColor`.
    GridColor,
    /// Maps to AppKit's `windowBackgroundColor`.
    WindowBackgroundColor,
    /// Maps to AppKit's `controlBackgroundColor`.
    ControlBackgroundColor,
    /// Maps to AppKit's `underPageBackgroundColor`.
    UnderPageBackgroundColor,
    /// Maps to AppKit's `selectedContentBackgroundColor`.
    SelectedContentBackgroundColor,
    /// Maps to AppKit's `unemphasizedSelectedContentBackgroundColor`.
    UnemphasizedSelectedContentBackgroundColor,
    /// Maps to AppKit's `textColor`.
    TextColor,
    /// Maps to AppKit's `textBackgroundColor`.
    TextBackgroundColor,
    /// Maps to AppKit's `selectedTextBackgroundColor`.
    SelectedTextBackgroundColor,
    /// Maps to AppKit's `controlColor`.
    ControlColor,
    /// Maps to AppKit's `controlTextColor`.
    ControlTextColor,
    /// Maps to AppKit's `disabledControlTextColor`.
    DisabledControlTextColor,
    /// Maps to AppKit's `keyboardFocusIndicatorColor`.
    KeyboardFocusIndicatorColor,
    /// Maps to AppKit's `selectedControlColor`.
    SelectedControlColor,
    /// Maps to AppKit's `alternateSelectedControlTextColor`.
    AlternateSelectedControlTextColor,
    /// Maps to AppKit's `systemFillColor`.
    SystemFillColor,
    /// Maps to AppKit's `secondarySystemFillColor`.
    SecondarySystemFillColor,
    /// Maps to AppKit's `tertiarySystemFillColor`.
    TertiarySystemFillColor,
    /// Maps to AppKit's `quaternarySystemFillColor`.
    QuaternarySystemFillColor,
    /// Maps to AppKit's `quinarySystemFillColor`.
    QuinarySystemFillColor,
    /// Maps to AppKit's `controlAccentColor`.
    ControlAccentColor,
    /// Maps to AppKit's `systemRedColor`.
    SystemRedColor,
    /// Maps to AppKit's `systemOrangeColor`.
    SystemOrangeColor,
    /// Maps to AppKit's `systemYellowColor`.
    SystemYellowColor,
    /// Maps to AppKit's `systemGreenColor`.
    SystemGreenColor,
    /// Maps to AppKit's `systemMintColor`.
    SystemMintColor,
    /// Maps to AppKit's `systemTealColor`.
    SystemTealColor,
    /// Maps to AppKit's `systemCyanColor`.
    SystemCyanColor,
    /// Maps to AppKit's `systemBlueColor`.
    SystemBlueColor,
    /// Maps to AppKit's `systemIndigoColor`.
    SystemIndigoColor,
    /// Maps to AppKit's `systemPurpleColor`.
    SystemPurpleColor,
    /// Maps to AppKit's `systemPinkColor`.
    SystemPinkColor,
    /// Maps to AppKit's `systemBrownColor`.
    SystemBrownColor,
    /// Maps to AppKit's `systemGrayColor`.
    SystemGrayColor,
}

/// Resolves an AppKit system color to concrete sRGB components for GPUI.
///
/// Resolve colors again after [`observe_effective_appearance`] reports a
/// change. AppKit also incorporates current system color and accessibility
/// preferences when it resolves the requested color.
///
/// # Panics
///
/// Panics when called off the main thread, on systems before macOS 26.1, or if
/// AppKit cannot convert the requested system color to sRGB.
pub fn resolve_system_color(color: SystemColor, appearance: Appearance) -> Rgba {
    let mtm = crate::platform::assert_supported_os();
    crate::platform::resolve_system_color(color, appearance, mtm)
}

/// Returns the non-vibrant appearance corresponding to a GPUI window's
/// effective AppKit appearance.
pub fn effective_appearance(window: &Window) -> Appearance {
    window.appearance().into()
}

/// Invokes a callback when a window's effective AppKit appearance changes.
///
/// The callback receives the corresponding non-vibrant appearance. Keep the
/// returned subscription alive and re-resolve every system color used by that
/// window when the callback runs.
pub fn observe_effective_appearance(
    window: &Window,
    mut callback: impl FnMut(Appearance, &mut Window, &mut App) + 'static,
) -> Subscription {
    window.observe_window_appearance(move |window, cx| {
        callback(effective_appearance(window), window, cx);
    })
}

/// Returns whether the system Reduce Transparency setting is enabled.
///
/// # Panics
///
/// Panics when called off the main thread or on systems before macOS 26.1.
pub fn reduce_transparency_enabled() -> bool {
    let mtm = crate::platform::assert_supported_os();
    crate::platform::reduce_transparency_enabled(mtm)
}

/// Returns whether the system Reduce Motion setting is enabled.
///
/// # Panics
///
/// Panics when called off the main thread or on systems before macOS 26.1.
pub fn reduce_motion_enabled() -> bool {
    let mtm = crate::platform::assert_supported_os();
    crate::platform::reduce_motion_enabled(mtm)
}

#[cfg(test)]
mod tests {
    use super::Appearance;

    #[test]
    fn gpui_vibrant_appearances_map_to_non_vibrant_resolution() {
        assert_eq!(
            Appearance::from(gpui::WindowAppearance::VibrantLight),
            Appearance::Aqua
        );
        assert_eq!(
            Appearance::from(gpui::WindowAppearance::VibrantDark),
            Appearance::DarkAqua
        );
    }
}
