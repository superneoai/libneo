#![deny(clippy::undocumented_unsafe_blocks)]

//! libneo builds native macOS application windows with GPUI and AppKit.

mod lifecycle;
mod native_views;
mod platform;
pub mod theme;
pub mod window;

pub use lifecycle::{NativeRoot, install};
