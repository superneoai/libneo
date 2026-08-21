//! Bridges concrete GPUI colors and AppKit colors.

use std::cell::Cell;

use block2::StackBlock;
use gpui::Rgba;
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{
    NSAppearance, NSAppearanceNameAccessibilityHighContrastAqua,
    NSAppearanceNameAccessibilityHighContrastDarkAqua, NSAppearanceNameAqua,
    NSAppearanceNameDarkAqua, NSColor, NSColorSpace, NSWorkspace,
};

use crate::appearance::{Appearance, SystemColor};

pub(crate) fn rgba_to_ns_color(color: Rgba) -> Retained<NSColor> {
    NSColor::colorWithSRGBRed_green_blue_alpha(
        color.r.into(),
        color.g.into(),
        color.b.into(),
        color.a.into(),
    )
}

pub(crate) fn resolve_system_color(
    color: SystemColor,
    appearance: Appearance,
    _mtm: MainThreadMarker,
) -> Rgba {
    let appearance = named_appearance(appearance);
    let resolved = Cell::new(None);
    let block = StackBlock::new(|| {
        resolved.set(Some(ns_color_to_rgba(&appkit_color(color))));
    });
    appearance.performAsCurrentDrawingAppearance(&block);
    resolved
        .into_inner()
        .expect("AppKit must invoke the appearance drawing block")
}

pub(crate) fn reduce_transparency_enabled(_mtm: MainThreadMarker) -> bool {
    NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceTransparency()
}

pub(crate) fn reduce_motion_enabled(_mtm: MainThreadMarker) -> bool {
    NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceMotion()
}

fn named_appearance(appearance: Appearance) -> Retained<NSAppearance> {
    let increased_contrast =
        NSWorkspace::sharedWorkspace().accessibilityDisplayShouldIncreaseContrast();
    // SAFETY: AppKit defines these immutable appearance names for the process lifetime.
    let name = unsafe {
        match (appearance, increased_contrast) {
            (Appearance::Aqua, false) => NSAppearanceNameAqua,
            (Appearance::Aqua, true) => NSAppearanceNameAccessibilityHighContrastAqua,
            (Appearance::DarkAqua, false) => NSAppearanceNameDarkAqua,
            (Appearance::DarkAqua, true) => NSAppearanceNameAccessibilityHighContrastDarkAqua,
        }
    };
    NSAppearance::appearanceNamed(name).expect("AppKit must provide Aqua and Dark Aqua appearances")
}

fn appkit_color(color: SystemColor) -> Retained<NSColor> {
    match color {
        SystemColor::LabelColor => NSColor::labelColor(),
        SystemColor::SecondaryLabelColor => NSColor::secondaryLabelColor(),
        SystemColor::TertiaryLabelColor => NSColor::tertiaryLabelColor(),
        SystemColor::QuaternaryLabelColor => NSColor::quaternaryLabelColor(),
        SystemColor::QuinaryLabelColor => NSColor::quinaryLabelColor(),
        SystemColor::PlaceholderTextColor => NSColor::placeholderTextColor(),
        SystemColor::LinkColor => NSColor::linkColor(),
        SystemColor::SeparatorColor => NSColor::separatorColor(),
        SystemColor::GridColor => NSColor::gridColor(),
        SystemColor::WindowBackgroundColor => NSColor::windowBackgroundColor(),
        SystemColor::ControlBackgroundColor => NSColor::controlBackgroundColor(),
        SystemColor::UnderPageBackgroundColor => NSColor::underPageBackgroundColor(),
        SystemColor::SelectedContentBackgroundColor => NSColor::selectedContentBackgroundColor(),
        SystemColor::UnemphasizedSelectedContentBackgroundColor => {
            NSColor::unemphasizedSelectedContentBackgroundColor()
        }
        SystemColor::TextColor => NSColor::textColor(),
        SystemColor::TextBackgroundColor => NSColor::textBackgroundColor(),
        SystemColor::SelectedTextBackgroundColor => NSColor::selectedTextBackgroundColor(),
        SystemColor::ControlColor => NSColor::controlColor(),
        SystemColor::ControlTextColor => NSColor::controlTextColor(),
        SystemColor::DisabledControlTextColor => NSColor::disabledControlTextColor(),
        SystemColor::KeyboardFocusIndicatorColor => NSColor::keyboardFocusIndicatorColor(),
        SystemColor::SelectedControlColor => NSColor::selectedControlColor(),
        SystemColor::AlternateSelectedControlTextColor => {
            NSColor::alternateSelectedControlTextColor()
        }
        SystemColor::SystemFillColor => NSColor::systemFillColor(),
        SystemColor::SecondarySystemFillColor => NSColor::secondarySystemFillColor(),
        SystemColor::TertiarySystemFillColor => NSColor::tertiarySystemFillColor(),
        SystemColor::QuaternarySystemFillColor => NSColor::quaternarySystemFillColor(),
        SystemColor::QuinarySystemFillColor => NSColor::quinarySystemFillColor(),
        SystemColor::ControlAccentColor => NSColor::controlAccentColor(),
        SystemColor::SystemRedColor => NSColor::systemRedColor(),
        SystemColor::SystemOrangeColor => NSColor::systemOrangeColor(),
        SystemColor::SystemYellowColor => NSColor::systemYellowColor(),
        SystemColor::SystemGreenColor => NSColor::systemGreenColor(),
        SystemColor::SystemMintColor => NSColor::systemMintColor(),
        SystemColor::SystemTealColor => NSColor::systemTealColor(),
        SystemColor::SystemCyanColor => NSColor::systemCyanColor(),
        SystemColor::SystemBlueColor => NSColor::systemBlueColor(),
        SystemColor::SystemIndigoColor => NSColor::systemIndigoColor(),
        SystemColor::SystemPurpleColor => NSColor::systemPurpleColor(),
        SystemColor::SystemPinkColor => NSColor::systemPinkColor(),
        SystemColor::SystemBrownColor => NSColor::systemBrownColor(),
        SystemColor::SystemGrayColor => NSColor::systemGrayColor(),
    }
}

fn ns_color_to_rgba(color: &NSColor) -> Rgba {
    let color = color
        .colorUsingColorSpace(&NSColorSpace::sRGBColorSpace())
        .expect("the AppKit system color must convert to sRGB");
    let (mut red, mut green, mut blue, mut alpha) = (0.0, 0.0, 0.0, 0.0);
    // SAFETY: each pointer refers to a live CGFloat output for the duration of the call.
    unsafe {
        color.getRed_green_blue_alpha(&mut red, &mut green, &mut blue, &mut alpha);
    }
    Rgba {
        r: red as f32,
        g: green as f32,
        b: blue as f32,
        a: alpha as f32,
    }
}
