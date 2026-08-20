# libneo-gpui

libneo-gpui integrates GPUI applications with AppKit on macOS.

GPUI draws the application content. AppKit draws the window chrome, the glass
effects, and the native text tables. libneo-gpui requires macOS 26.1 or later,
and uses public AppKit APIs.

## Features

- AppKit windows with a transparent title bar or a native toolbar.
- Window backgrounds with `NSVisualEffectView` materials.
- Glass effects that take their position from GPUI layout.
- Native text tables with the system scroll edge effect.
- Themes that follow the system appearance.
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

`libneo::window::run` uses this same lifecycle automatically.

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
