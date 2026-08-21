use gpui::{Context, IntoElement, Render, Window, div, prelude::*, px, rgb, rgba};
use libneo_gpui::glass::{
    GlassEffectConfiguration, GlassEffectGroup, GlassEffectStyle, glass_effect,
};
use libneo_gpui::window::{
    WindowBackground, WindowBackgroundAppearance, WindowBuilder, WindowChrome, WindowCornerRadius,
    run,
};

struct Example {
    count: usize,
}

impl Render for Example {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let group = GlassEffectGroup::new("example.cards", px(12.0));
        let button = glass_effect(
            "example.increment-material",
            GlassEffectConfiguration {
                style: GlassEffectStyle::Clear,
                corner_radius: px(12.0),
                tint: Some(rgba(0x007affcc)),
                group: Some(group.clone()),
            },
        )
        .child(
            div()
                .id("increment")
                .px_4()
                .py_2()
                .text_color(rgb(0xffffff))
                .cursor_pointer()
                .on_click(cx.listener(|this, _, _, cx| {
                    this.count += 1;
                    cx.notify();
                }))
                .child("Increment"),
        );

        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .text_color(rgb(0x1c1c1e))
            .child(
                glass_effect(
                    "example.glass-card",
                    GlassEffectConfiguration {
                        style: GlassEffectStyle::Regular,
                        corner_radius: px(28.0),
                        tint: None,
                        group: Some(group),
                    },
                )
                .w(px(360.0))
                .h(px(220.0))
                .p_8()
                .flex()
                .flex_col()
                .items_start()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(div().text_size(px(24.0)).child("Application content"))
                        .child(format!("The GPUI control has fired {} times.", self.count)),
                )
                .child(button),
            )
    }
}

fn main() {
    let window = WindowBuilder::new(
        "Glass Content Example",
        (720.0, 480.0),
        (560.0, 360.0),
        (12.0, 12.0),
        WindowCornerRadius::System,
        WindowChrome::TransparentTitleBar,
        WindowBackground::Standard,
        WindowBackgroundAppearance::Transparent,
    );

    run(window, |_| Example { count: 0 });
}
