use gpui::{
    App, Context, FocusHandle, Focusable, IntoElement, Render, Window, actions, div, prelude::*,
    rgb,
};
use libneo_gpui::toolbar::{
    Toolbar, ToolbarConfiguration, ToolbarDisplayMode, ToolbarItem, ToolbarStyle, ToolbarSystemItem,
};
use libneo_gpui::window::{
    WindowBackground, WindowBackgroundAppearance, WindowBuilder, WindowChrome, run,
};

const EMPTY: &str = "empty";

actions!(toolbar_example, [Fire]);

struct Example {
    fired: bool,
    focus: FocusHandle,
}

impl Example {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            fired: false,
            focus: cx.focus_handle(),
        }
    }

    fn fire(&mut self, _: &Fire, _: &mut Window, cx: &mut Context<Self>) {
        self.fired = true;
        cx.notify();
    }
}

impl Focusable for Example {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for Example {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.focus.is_focused(window) {
            self.focus.focus(window, cx);
        }
        div()
            .track_focus(&self.focus)
            .on_action(cx.listener(Self::fire))
            .flex()
            .size_full()
            .items_center()
            .justify_center()
            .bg(rgb(0xf2f2f7))
            .text_color(rgb(0x1c1c1e))
            .child(if self.fired {
                "Toolbar action fired"
            } else {
                "Toolbar action ready"
            })
    }
}

fn toolbar_configuration() -> ToolbarConfiguration {
    ToolbarConfiguration {
        display_mode: ToolbarDisplayMode::IconAndLabel,
        style: ToolbarStyle::Unified,
        autosaves_configuration: false,
        allows_user_customization: false,
    }
}

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_else(|| EMPTY.to_owned());
    let toolbar = if mode == EMPTY {
        Toolbar::new("example.empty-toolbar", toolbar_configuration())
    } else {
        Toolbar::new("example.declared-toolbar", toolbar_configuration()).items([
            ToolbarItem::action("example.fire", "Fire Action", Fire).symbol("bolt.fill"),
            ToolbarItem::system(ToolbarSystemItem::FlexibleSpace),
            ToolbarItem::action("example.disabled", "Disabled", Fire)
                .symbol("nosign")
                .enabled(false),
        ])
    };
    let window = WindowBuilder::new(
        format!("Toolbar Example — {mode}"),
        (720.0, 440.0),
        (640.0, 320.0),
        (12.0, 12.0),
        WindowChrome::Toolbar(toolbar),
        WindowBackground::Standard,
        WindowBackgroundAppearance::Opaque,
    );

    run(window, Example::new);
}
