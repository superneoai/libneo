//! Opens and configures the window.

use gpui::{
    App, AppContext, Size, TitlebarOptions, WindowBounds, WindowKind, WindowOptions, point, size,
};

use crate::toolbar::Toolbar;

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
    /// Returns the title bar transparency.
    const fn title_bar_is_transparent(&self) -> bool {
        matches!(self, Self::TransparentTitleBar)
    }
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
    background_appearance: WindowBackgroundAppearance,
    background: WindowBackground,
    chrome: WindowChrome,
}

impl WindowBuilder {
    /// Creates a builder from caller-supplied window presentation.
    ///
    /// The controls position applies only to transparent title bars because
    /// AppKit positions controls for toolbar chrome. The background appearance
    /// applies only to standard backgrounds; visual-effect backgrounds require
    /// a transparent GPUI surface so the AppKit material remains visible.
    pub fn new(
        title: impl Into<String>,
        size: (f32, f32),
        minimum_size: (f32, f32),
        window_controls_position: (f32, f32),
        chrome: WindowChrome,
        background: WindowBackground,
        background_appearance: WindowBackgroundAppearance,
    ) -> Self {
        Self {
            title: title.into(),
            size,
            minimum_size,
            window_controls_position,
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
