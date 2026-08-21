use gpui::{Context, IntoElement, Render, Window, div, prelude::*, rgb};
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
            .child("Caller-defined window corners");
        if self.opaque_background {
            root.bg(rgb(0xf2f2f7))
        } else {
            root
        }
    }
}

fn main() {
    let visual_effect = std::env::args().nth(1).as_deref() == Some("visual-effect");
    let background = if visual_effect {
        WindowBackground::VisualEffect(VisualEffectMaterial::UnderWindowBackground)
    } else {
        WindowBackground::Standard
    };
    let window = WindowBuilder::new(
        "Window Corners Example",
        (720.0, 440.0),
        (640.0, 320.0),
        (12.0, 12.0),
        WindowCornerRadius::Fixed(24.0),
        WindowChrome::TransparentTitleBar,
        background,
        WindowBackgroundAppearance::Opaque,
    );

    run(window, move |_| Example {
        opaque_background: !visual_effect,
    });
}
