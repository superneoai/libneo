# Changelog

This file records umbrella and facade changes. Focused package changelogs record
adapter changes.

This file follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0-alpha.1]

### Added

- A dependency-neutral `libneo` facade with an empty default feature set.
- An opt-in `gpui` feature that re-exports the `libneo-gpui` adapter.
- The focused `libneo-gpui` package for the GPUI and AppKit APIs.
- GPUI-backed application menu bars through the facade's `menu` module.
- Caller-defined native toolbars through the facade's `toolbar` module.
