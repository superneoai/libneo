# libneo

libneo is an umbrella for focused native application integration packages. The
`libneo` facade has no dependencies or platform requirements by default.
Adapters are opt-in features.

## Packages

| Package | Purpose |
| --- | --- |
| `libneo` | Dependency-neutral facade; its default feature set is empty. |
| `libneo-gpui` | GPUI and AppKit integration for macOS 26.1 and later. |

Enable `libneo`'s `gpui` feature to expose `libneo::install`,
`libneo::NativeRoot`, and the public modules for windows,
glass, tables, and layers. The macOS 26.1 deployment-target check runs
only when you select `libneo-gpui`.

## Consumer setup

This Git repository distributes the packages. The packages do not go to
crates.io.

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

Cargo finds `libneo` below this virtual workspace root. Patch sections apply at
the consumer workspace root only. Pin the `libneo`, `ztracing`, and `block`
entries to the same 40-character libneo commit SHA.

GPUI consumers must set the deployment target in their workspace at
`.cargo/config.toml`:

```toml
[env]
MACOSX_DEPLOYMENT_TARGET = "26.1"
```

See [`crates/libneo-gpui`](crates/libneo-gpui/README.md) for adapter features,
compatibility, and usage.

## Build

```sh
cargo fmt --all --check
cargo build --workspace --locked
cargo check --workspace --locked
cargo tree -p libneo --no-default-features --locked
cargo tree -p libneo --features gpui --locked
cargo check --workspace --locked --target x86_64-apple-darwin
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test --manifest-path vendor/block/Cargo.toml
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --workspace --no-deps --locked
cargo deny --locked check
taplo lint $(git ls-files '*.toml' ':!:Cargo.lock')
taplo fmt --check $(git ls-files '*.toml' ':!:Cargo.lock')
markdownlint $(git ls-files '*.md')
actionlint -no-color .github/workflows/ci.yml
```

## Dependency licenses

GPUI declares Apache-2.0 and links `ztracing`, which Zed publishes under
`GPL-3.0-or-later`. Zed tracks this state in
<https://github.com/zed-industries/zed/issues/55470>.

This repository supplies a permissive `ztracing` compatibility crate. It also
patches the MIT-licensed `block 0.1.6` crate for Rust compatibility.

## License

Copyright (c) 2026 ACTUAL LTD.

libneo is available under either of these licenses:

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)
