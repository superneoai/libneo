#![deny(clippy::undocumented_unsafe_blocks)]

//! libneo integrates GPUI applications with AppKit on macOS.
//!
//! GPUI draws the application content. AppKit draws the window chrome, the
//! glass effects, and the native text tables. libneo targets macOS 26.1
//! and later, and uses public AppKit APIs.
//!
//! Existing GPUI applications call [`install`] before opening windows and wrap
//! every window root that uses native elements in [`NativeRoot`].

pub mod glass;
mod lifecycle;
mod native_views;
mod platform;
pub mod table;
pub mod theme;
pub mod window;

pub use lifecycle::{NativeRoot, install};
