//! Opens and configures the window.

use gpui::{
    App, AppContext, Size, TitlebarOptions, WindowBounds, WindowKind, WindowOptions, point, size,
};

pub use gpui::colors::DefaultColors;
pub use gpui::{
    Context, IntoElement, ParentElement, Render, Rgba, Styled, Window, WindowBackgroundAppearance,
    div, px, rgba,
};

const DEFAULT_SIZE: (f32, f32) = (960.0, 640.0);
const DEFAULT_MINIMUM_SIZE: (f32, f32) = (640.0, 480.0);
const DEFAULT_WINDOW_CONTROLS_POSITION: (f32, f32) = (12.0, 12.0);

/// Selects an AppKit `NSVisualEffectMaterial`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum VisualEffectMaterial {
    /// Uses the material for content below the window background.
    UnderWindowBackground,
    /// Uses the heads-up display material.
    #[default]
    HudWindow,
    /// Uses the sidebar material.
    Sidebar,
}

/// Selects the window chrome.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum WindowChrome {
    /// Shows a transparent title bar. Content fills the window.
    #[default]
    TransparentTitleBar,
    /// Shows a native toolbar in an opaque title bar. Content passes below the
    /// toolbar material.
    Toolbar,
}

impl WindowChrome {
    /// Returns the title bar transparency.
    const fn title_bar_is_transparent(self) -> bool {
        matches!(self, Self::TransparentTitleBar)
    }
}

/// Selects the window background.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum WindowBackground {
    /// Uses the GPUI window background.
    #[default]
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
    background_appearance: WindowBackgroundAppearance,
    background: WindowBackground,
    chrome: WindowChrome,
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self {
            title: "neo".to_owned(),
            size: DEFAULT_SIZE,
            minimum_size: DEFAULT_MINIMUM_SIZE,
            window_controls_position: DEFAULT_WINDOW_CONTROLS_POSITION,
            background_appearance: WindowBackgroundAppearance::Opaque,
            background: WindowBackground::Standard,
            chrome: WindowChrome::TransparentTitleBar,
        }
    }
}

impl WindowBuilder {
    /// Creates a builder with default values.
    pub fn new() -> Self {
        Self::default()
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

    /// Sets the GPUI window background for [`WindowBackground::Standard`].
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
/// This function panics off the main thread, on systems before macOS 26.1, and
/// when the window fails to open.
pub fn run<V>(window: WindowBuilder, build_root: impl FnOnce(&mut Context<V>) -> V + 'static)
where
    V: Render + 'static,
{
    gpui_platform::application().run(move |cx: &mut App| {
        crate::install(cx);
        let options = window.window_options(cx);
        let background = window.background;
        let chrome = window.chrome;

        cx.open_window(options, move |gpui_window, cx| {
            let mtm = crate::lifecycle::main_thread_marker(cx);
            crate::native_views::configure_window_chrome(gpui_window, chrome, mtm, cx)
                .expect("the window chrome must apply");
            crate::native_views::configure_window_background(gpui_window, background, mtm, cx)
                .expect("the window background must apply");
            let content = cx.new(build_root);
            cx.new(|_| crate::NativeRoot::new(content))
        })
        .expect("the window must open");
        cx.activate(true);
    });
}
