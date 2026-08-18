//! Dependency-neutral access to libneo integration packages.
//!
//! The default feature set has no dependencies. Enable `gpui` to expose the
//! GPUI and AppKit integration from `libneo-gpui`.

#[cfg(feature = "gpui")]
pub use libneo_gpui::*;
