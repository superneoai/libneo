# libneo-gpui

libneo-gpui integrates GPUI applications with AppKit on macOS.

GPUI draws the application content. AppKit draws the window chrome, the glass
effects, and the native text tables. libneo-gpui requires macOS 26.1 or later,
and uses public AppKit APIs.

## Features

- AppKit windows with a transparent title bar or a native toolbar.
- Caller-defined corner radii for transparent-title-bar windows.
- AppKit system-color resolution for caller-owned GPUI colors.
- Window backgrounds with `NSVisualEffectView` materials.
- Glass effects that take their position from GPUI layout.
- Native text tables with the system scroll edge effect.
- Native application, Window, and Help menus backed by GPUI actions.
- Overlay placement.

## Consumer setup

This Git repository distributes libneo-gpui. The package does not go to
crates.io. The `libneo` facade exposes it through the opt-in `gpui` feature.

libneo-gpui supports only macOS 26.1 and later. Set the deployment target in the
**consumer workspace** at `.cargo/config.toml`:

```toml
[env]
MACOSX_DEPLOYMENT_TARGET = "26.1"
```

The dependency build script rejects a missing, invalid, or lower macOS target.
libneo-gpui supports macOS only.

### GPUI compatibility

| libneo-gpui version | Zed revision |
| --- | --- |
| 0.1.0-alpha.1 | `bc538def4545534201bbfcac4e95ac34ea6501b6` |

All direct Zed dependencies in the consumer workspace must use this same Git
URL and revision. Existing GPUI applications can add the dependencies and the
required workspace-root patches as follows:

```toml
[dependencies.gpui]
git = "https://github.com/zed-industries/zed"
rev = "bc538def4545534201bbfcac4e95ac34ea6501b6"

[dependencies.gpui_platform]
git = "https://github.com/zed-industries/zed"
rev = "bc538def4545534201bbfcac4e95ac34ea6501b6"
features = ["font-kit"]

[dependencies.libneo]
git = "https://github.com/superneoai/libneo"
rev = "<libneo-commit-sha>"
default-features = false
features = ["gpui"]

[patch."https://github.com/zed-industries/zed".ztracing]
git = "https://github.com/superneoai/libneo"
rev = "<libneo-commit-sha>"

[patch."https://github.com/zed-industries/font-kit".zed-font-kit]
version = "=0.14.1-zed"

[patch.crates-io.block]
git = "https://github.com/superneoai/libneo"
rev = "<libneo-commit-sha>"
```

Cargo applies a patch section at the workspace root only. Keep these patches in
the consumer workspace root. Pin the `libneo`, `ztracing`, and `block` entries
to the same 40-character libneo commit SHA; the font-kit patch selects Zed's
matching crates.io release.

Install libneo-gpui through the facade before opening windows and wrap each
window's existing root
entity in `NativeRoot`:

```rust
use gpui::{App, AppContext as _, WindowOptions};
use libneo::{NativeRoot, install};

fn open_main_window(cx: &mut App) {
    cx.open_window(WindowOptions::default(), |_window, cx| {
        let root = cx.new(|cx| ExistingRoot::new(cx));
        cx.new(|_| NativeRoot::new(root))
    })
    .expect("the main window must open");
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        install(cx);
        open_main_window(cx);
        cx.activate(true);
    });
}
```

`libneo::window::run` uses this same lifecycle automatically. After calling
`install`, use `WindowBuilder::open` to apply the same presentation to every
additional window. Installation does not initialize GPUI colors or application
theme state; the consumer owns both.

Use `appearance::resolve_system_color` to obtain a concrete `Rgba` for a
caller-selected AppKit `SystemColor` and `Appearance`. Resolution deliberately
uses Aqua and Dark Aqua rather than the vibrant appearances. GPUI does not
participate in AppKit vibrancy, so drawing a vibrant color directly would use a
value pre-compensated for a blend that never occurs and would reduce contrast.
Keep the subscription from `appearance::observe_effective_appearance` and
re-resolve the colors used by that window when its callback runs. The same
module reports the current Reduce Transparency and Reduce Motion settings.

Install a menu bar after libneo initialization. Standard constructors supply the
macOS application, Window, and Help menu structure; custom menus and submenus
use the same compact builders. Menu commands and shortcuts dispatch GPUI
actions, so focused GPUI handlers also control command availability.

```rust
use libneo::menu::{Menu, MenuBar};

MenuBar::new()
    .menus([Menu::application("Example"), Menu::window(), Menu::help()])
    .install(cx);
```

Register a handler for `libneo::menu::Settings` when the application has a
settings interface. Without a handler, GPUI leaves the standard Settings item
visible but unavailable.

Build native toolbars from caller-owned identifiers, presentation, labels, SF
Symbols, and GPUI actions. System items cover spacing and standard AppKit
commands. An empty item list produces an empty toolbar. Window construction
requires every presentation choice; no title, dimensions, control position,
corner radius, chrome, or background is inferred.

```rust
use libneo::toolbar::{
    Toolbar, ToolbarConfiguration, ToolbarDisplayMode, ToolbarItem,
    ToolbarStyle, ToolbarSystemItem,
};
use libneo::window::{
    WindowBackground, WindowBackgroundAppearance, WindowBuilder, WindowChrome,
    WindowCornerRadius,
};

let toolbar = Toolbar::new(
    "example.main-toolbar",
    ToolbarConfiguration {
        display_mode: ToolbarDisplayMode::IconAndLabel,
        style: ToolbarStyle::Unified,
        autosaves_configuration: false,
        allows_user_customization: false,
    },
)
.items([
    ToolbarItem::action("example.search", "Search", Search)
        .symbol("magnifyingglass"),
    ToolbarItem::system(ToolbarSystemItem::FlexibleSpace),
]);
let window = WindowBuilder::new(
    "Example",
    (960.0, 640.0),
    (640.0, 480.0),
    (12.0, 12.0),
    WindowCornerRadius::System,
    WindowChrome::Toolbar(toolbar),
    WindowBackground::Standard,
    WindowBackgroundAppearance::Opaque,
);
```

Toolbar action availability follows the focused GPUI dispatch path. Use
`ToolbarItem::enabled(false)` for an explicitly unavailable item.

Use `WindowCornerRadius::Fixed(48.0)` with either chrome variant to shape the
outer window and its system shadow. The adapter makes an `NSVisualEffectView`
the window content root, applies a resizable rounded mask image, clips its
subviews to the same radius, clears the native window background, and asks
AppKit to recompute the shadow. A visual-effect background retains the exact
semantic material selected by the caller; a standard opaque GPUI surface hides
the mask host's material. AppKit's system frame mask remains a lower bound, so
a caller can make a window rounder but not squarer.

The minimum accepted fixed radius depends on the chrome presentation:

| Chrome | Minimum radius |
| --- | ---: |
| Transparent title bar | 16 pt |
| Automatic toolbar | 26 pt |
| Expanded toolbar | 16 pt |
| Preferences toolbar | 16 pt |
| Unified toolbar | 26 pt |
| Compact unified toolbar | 20 pt |

`WindowBuilder::open` rejects non-finite values and values below the applicable
named floor instead of silently accepting a system-clamped result. GPUI creates
its native content tree before the open callback and does not replace it later;
the adapter applies chrome first, installs the material root last, and retains
that root until the window closes. It does not restore the previous content
view during teardown.

Run `cargo run -p libneo-gpui --example window_corners` to inspect the fixed
radius with a standard background, append `-- visual-effect` for a native
material background, or append `-- toolbar` for a unified native toolbar. Run
`cargo run -p libneo-gpui --example toolbar -- empty` (or `declared`) to inspect
the toolbar conformance example. Run
`cargo run -p libneo-gpui --example system_colors` to resolve sample system
colors in both supported appearances and report the accessibility settings.

## Build

```sh
cargo fmt --all --check
cargo build --workspace --locked
cargo build -p libneo --features gpui --locked
cargo check --workspace --locked
cargo check -p libneo --features gpui --locked
cargo check --workspace --locked --target x86_64-apple-darwin
cargo check -p libneo --features gpui --locked --target x86_64-apple-darwin
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy -p libneo --all-targets --features gpui --locked -- -D warnings
cargo test --workspace --locked
cargo test -p libneo --features gpui --locked
cargo test --manifest-path vendor/block/Cargo.toml
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --workspace --no-deps --locked
RUSTDOCFLAGS="-D warnings -D missing-docs" \
  cargo doc -p libneo --features gpui --no-deps --locked
cargo deny --locked check
taplo lint $(git ls-files '*.toml' ':!:Cargo.lock')
taplo fmt --check $(git ls-files '*.toml' ':!:Cargo.lock')
markdownlint $(git ls-files '*.md')
actionlint -no-color .github/workflows/ci.yml
```

## Dependency licenses

GPUI declares Apache-2.0, and links `ztracing`, which Zed publishes under
`GPL-3.0-or-later`. Zed tracks this state in
<https://github.com/zed-industries/zed/issues/55470>.

This repository supplies a permissive `ztracing` compatibility crate, so its
own builds carry no GPL terms. It also patches the MIT-licensed `block 0.1.6`
crate for Rust compatibility.

## License

Copyright (c) 2026 ACTUAL LTD.

libneo-gpui is available under either of these licenses:

- [MIT License](../../LICENSE-MIT)
- [Apache License 2.0](../../LICENSE-APACHE)
