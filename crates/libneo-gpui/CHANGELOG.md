# libneo-gpui changelog

This file records notable changes to libneo-gpui.

This file follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Caller-controlled outer corner radii and shape-following shadows for titled
  windows with transparent title bars or native toolbars.

### Changed

- Window, toolbar, glass, table, and overlay construction now requires every
  application-owned presentation choice.

### Removed

- The application-specific theme palette and automatic GPUI color
  initialization.

## [0.1.0-alpha.1]

### Added

- AppKit windows with a transparent title bar or a native toolbar.
- Window backgrounds with `NSVisualEffectView` materials.
- Glass effects with grouping and native content.
- Native text tables.
- Themes that follow the system appearance.
- Application menu bars with GPUI actions, shortcuts, state, submenus, and
  macOS standard menus.
- Caller-defined native toolbars with GPUI actions and AppKit system items.
- Overlay placement.
