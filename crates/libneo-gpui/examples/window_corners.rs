use gpui::{Context, IntoElement, Render, Window, div, prelude::*, rgb};
use libneo_gpui::toolbar::{Toolbar, ToolbarConfiguration, ToolbarDisplayMode, ToolbarStyle};
use libneo_gpui::window::{
    VisualEffectMaterial, WindowBackground, WindowBackgroundAppearance, WindowBuilder,
    WindowChrome, WindowCornerRadius, run,
};

struct Example {
    opaque_background: bool,
}

impl Render for Example {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let root = div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .child("Caller-defined titled-window corners");
        if self.opaque_background {
            root.bg(rgb(0xf2f2f7))
        } else {
            root
        }
    }
}

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_default();
    let visual_effect = mode == "visual-effect";
    let background = if visual_effect {
        WindowBackground::VisualEffect(VisualEffectMaterial::UnderWindowBackground)
    } else {
        WindowBackground::Standard
    };
    let chrome = if mode == "toolbar" {
        WindowChrome::Toolbar(Toolbar::new(
            "example.window-corners-toolbar",
            ToolbarConfiguration {
                display_mode: ToolbarDisplayMode::System,
                style: ToolbarStyle::Unified,
                autosaves_configuration: false,
                allows_user_customization: false,
            },
        ))
    } else {
        WindowChrome::TransparentTitleBar
    };
    let window = WindowBuilder::new(
        "Window Corners Example",
        (720.0, 440.0),
        (640.0, 320.0),
        (12.0, 12.0),
        WindowCornerRadius::Fixed(32.0),
        chrome,
        background,
        WindowBackgroundAppearance::Opaque,
    );

    run(window, move |_| Example {
        opaque_background: !visual_effect,
    });
}
