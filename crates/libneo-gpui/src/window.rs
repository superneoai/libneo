//! Opens and configures the window.

use gpui::{
    App, AppContext, Size, TitlebarOptions, WindowBounds, WindowKind, WindowOptions, point, size,
};

use crate::toolbar::{Toolbar, ToolbarStyle};

pub use gpui::{
    Context, IntoElement, ParentElement, Render, Rgba, Styled, Window, WindowBackgroundAppearance,
    div, px, rgba,
};

/// Selects an AppKit `NSVisualEffectMaterial`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VisualEffectMaterial {
    /// Uses the material for content below the window background.
    UnderWindowBackground,
    /// Uses the heads-up display material.
    HudWindow,
    /// Uses the sidebar material.
    Sidebar,
}

/// Selects the window chrome.
#[derive(Clone, Debug)]
pub enum WindowChrome {
    /// Shows a transparent title bar. Content fills the window.
    TransparentTitleBar,
    /// Shows the supplied native toolbar in an opaque title bar. Content passes
    /// below the toolbar material.
    Toolbar(Toolbar),
}

impl WindowChrome {
    const fn title_bar_is_transparent(&self) -> bool {
        matches!(self, Self::TransparentTitleBar)
    }

    const fn fixed_corner_radius_floor(&self) -> f32 {
        match self {
            Self::TransparentTitleBar
            | Self::Toolbar(Toolbar {
                configuration:
                    crate::toolbar::ToolbarConfiguration {
                        style: ToolbarStyle::Expanded | ToolbarStyle::Preference,
                        ..
                    },
                ..
            }) => 16.0,
            Self::Toolbar(Toolbar {
                configuration:
                    crate::toolbar::ToolbarConfiguration {
                        style: ToolbarStyle::UnifiedCompact,
                        ..
                    },
                ..
            }) => 20.0,
            Self::Toolbar(Toolbar {
                configuration:
                    crate::toolbar::ToolbarConfiguration {
                        style: ToolbarStyle::Automatic | ToolbarStyle::Unified,
                        ..
                    },
                ..
            }) => 26.0,
        }
    }

    const fn name(&self) -> &'static str {
        match self {
            Self::TransparentTitleBar => "transparent title-bar chrome",
            Self::Toolbar(Toolbar {
                configuration:
                    crate::toolbar::ToolbarConfiguration {
                        style: ToolbarStyle::Automatic,
                        ..
                    },
                ..
            }) => "automatic toolbar chrome",
            Self::Toolbar(Toolbar {
                configuration:
                    crate::toolbar::ToolbarConfiguration {
                        style: ToolbarStyle::Expanded,
                        ..
                    },
                ..
            }) => "expanded toolbar chrome",
            Self::Toolbar(Toolbar {
                configuration:
                    crate::toolbar::ToolbarConfiguration {
                        style: ToolbarStyle::Preference,
                        ..
                    },
                ..
            }) => "preferences toolbar chrome",
            Self::Toolbar(Toolbar {
                configuration:
                    crate::toolbar::ToolbarConfiguration {
                        style: ToolbarStyle::Unified,
                        ..
                    },
                ..
            }) => "unified toolbar chrome",
            Self::Toolbar(Toolbar {
                configuration:
                    crate::toolbar::ToolbarConfiguration {
                        style: ToolbarStyle::UnifiedCompact,
                        ..
                    },
                ..
            }) => "compact unified toolbar chrome",
        }
    }
}

fn validate_fixed_corner_radius(radius: f32, chrome: &WindowChrome) -> Result<(), String> {
    let minimum = chrome.fixed_corner_radius_floor();
    if radius.is_finite() && radius >= minimum {
        Ok(())
    } else {
        Err(format!(
            "the fixed window corner radius must be finite and at least {minimum} points for {}",
            chrome.name(),
        ))
    }
}

/// Selects the window corner treatment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WindowCornerRadius {
    /// Uses the corner treatment supplied by AppKit.
    System,
    /// Shapes the titled window and its shadow to the supplied radius in logical pixels.
    ///
    /// AppKit's frame mask is a lower bound. The minimum is 16 points for a
    /// transparent title bar, expanded toolbar, or preferences toolbar; 20
    /// points for a compact unified toolbar; and 26 points for an automatic or
    /// unified toolbar. [`run`] rejects lower, negative, or non-finite values.
    Fixed(f32),
}

/// Selects the window background.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindowBackground {
    /// Uses the GPUI window background.
    Standard,
    /// Shows a native `NSVisualEffectView` material below the GPUI content.
    VisualEffect(VisualEffectMaterial),
}

/// Configures the window that [`run`] opens.
#[derive(Clone, Debug)]
pub struct WindowBuilder {
    title: String,
    size: (f32, f32),
    minimum_size: (f32, f32),
    window_controls_position: (f32, f32),
    corner_radius: WindowCornerRadius,
    background_appearance: WindowBackgroundAppearance,
    background: WindowBackground,
    chrome: WindowChrome,
}

impl WindowBuilder {
    /// Creates a builder from caller-supplied window presentation.
    ///
    /// The controls position applies only to transparent title bars because
    /// AppKit positions controls for toolbar chrome. A fixed corner radius
    /// applies to all four outer corners in logical pixels. The background
    /// appearance applies only to standard backgrounds; visual-effect
    /// backgrounds require a transparent GPUI surface so the AppKit material
    /// remains visible.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: impl Into<String>,
        size: (f32, f32),
        minimum_size: (f32, f32),
        window_controls_position: (f32, f32),
        corner_radius: WindowCornerRadius,
        chrome: WindowChrome,
        background: WindowBackground,
        background_appearance: WindowBackgroundAppearance,
    ) -> Self {
        Self {
            title: title.into(),
            size,
            minimum_size,
            window_controls_position,
            corner_radius,
            background_appearance,
            background,
            chrome,
        }
    }

    /// Sets the window title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the start size in logical pixels.
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = (width, height);
        self
    }

    /// Sets the minimum size in logical pixels.
    pub fn minimum_size(mut self, width: f32, height: f32) -> Self {
        self.minimum_size = (width, height);
        self
    }

    /// Sets the position of the standard window controls in logical pixels for
    /// [`WindowChrome::TransparentTitleBar`].
    pub fn window_controls_position(mut self, x: f32, y: f32) -> Self {
        self.window_controls_position = (x, y);
        self
    }

    /// Sets the window corner treatment.
    pub fn corner_radius(mut self, corner_radius: WindowCornerRadius) -> Self {
        self.corner_radius = corner_radius;
        self
    }

    /// Sets the GPUI window background for [`WindowBackground::Standard`].
    ///
    /// Visual-effect backgrounds require a transparent GPUI surface so the
    /// AppKit material remains visible.
    pub fn background_appearance(mut self, appearance: WindowBackgroundAppearance) -> Self {
        self.background_appearance = appearance;
        self
    }

    /// Sets the window background.
    pub fn background(mut self, background: WindowBackground) -> Self {
        self.background = background;
        self
    }

    /// Sets the window chrome.
    pub fn chrome(mut self, chrome: WindowChrome) -> Self {
        self.chrome = chrome;
        self
    }

    fn window_options(&self, cx: &App) -> WindowOptions {
        let window_size = size(px(self.size.0), px(self.size.1));
        let minimum_size: Size<_> = size(px(self.minimum_size.0), px(self.minimum_size.1));
        let transparent = self.chrome.title_bar_is_transparent();

        WindowOptions {
            window_bounds: Some(WindowBounds::centered(window_size, cx)),
            titlebar: Some(TitlebarOptions {
                title: Some(self.title.clone().into()),
                appears_transparent: transparent,
                traffic_light_position: transparent.then(|| {
                    point(
                        px(self.window_controls_position.0),
                        px(self.window_controls_position.1),
                    )
                }),
            }),
            kind: WindowKind::Normal,
            window_background: match self.background {
                WindowBackground::Standard => self.background_appearance,
                WindowBackground::VisualEffect(_) => WindowBackgroundAppearance::Transparent,
            },
            window_min_size: Some(minimum_size),
            ..WindowOptions::default()
        }
    }
}

/// Runs the application and opens one window that shows `build_root`.
///
/// # Panics
///
/// This function panics off the main thread, on systems before macOS 26.1, when
/// a fixed corner radius is below the system floor for the selected chrome or
/// is not finite, and when the window fails to open.
pub fn run<V>(window: WindowBuilder, build_root: impl FnOnce(&mut Context<V>) -> V + 'static)
where
    V: Render + 'static,
{
    gpui_platform::application().run(move |cx: &mut App| {
        crate::install(cx);
        let options = window.window_options(cx);
        let background = window.background;
        let chrome = window.chrome;
        let corner_radius = match window.corner_radius {
            WindowCornerRadius::System => None,
            WindowCornerRadius::Fixed(radius) => {
                validate_fixed_corner_radius(radius, &chrome)
                    .unwrap_or_else(|error| panic!("{error}"));
                Some(radius)
            }
        };

        cx.open_window(options, move |gpui_window, cx| {
            let mtm = crate::lifecycle::main_thread_marker(cx);
            crate::native_views::configure_window_chrome(gpui_window, chrome, mtm, cx)
                .expect("the window chrome must apply");
            crate::native_views::configure_window_background(gpui_window, background, mtm, cx)
                .expect("the window background must apply");
            if let Some(corner_radius) = corner_radius {
                crate::native_views::configure_window_corner_radius(
                    gpui_window,
                    corner_radius,
                    mtm,
                    cx,
                )
                .expect("the window corner radius must apply");
            }
            let content = cx.new(build_root);
            cx.new(|_| crate::NativeRoot::new(content))
        })
        .expect("the window must open");
        cx.activate(true);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toolbar::{ToolbarConfiguration, ToolbarDisplayMode};

    fn toolbar_chrome(style: ToolbarStyle) -> WindowChrome {
        WindowChrome::Toolbar(Toolbar::new(
            "test.toolbar",
            ToolbarConfiguration {
                display_mode: ToolbarDisplayMode::System,
                style,
                autosaves_configuration: false,
                allows_user_customization: false,
            },
        ))
    }

    #[test]
    fn fixed_corner_radius_floors_follow_chrome_presentation() {
        let cases = [
            (WindowChrome::TransparentTitleBar, 16.0),
            (toolbar_chrome(ToolbarStyle::Automatic), 26.0),
            (toolbar_chrome(ToolbarStyle::Expanded), 16.0),
            (toolbar_chrome(ToolbarStyle::Preference), 16.0),
            (toolbar_chrome(ToolbarStyle::Unified), 26.0),
            (toolbar_chrome(ToolbarStyle::UnifiedCompact), 20.0),
        ];

        for (chrome, minimum) in cases {
            assert_eq!(chrome.fixed_corner_radius_floor(), minimum);
            assert!(validate_fixed_corner_radius(minimum, &chrome).is_ok());
            assert!(validate_fixed_corner_radius(minimum - 1.0, &chrome).is_err());
        }
    }

    #[test]
    fn fixed_corner_radius_rejects_non_finite_values() {
        for radius in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(
                validate_fixed_corner_radius(radius, &WindowChrome::TransparentTitleBar).is_err()
            );
        }
    }
}
