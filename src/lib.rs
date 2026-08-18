#![deny(clippy::undocumented_unsafe_blocks)]

//! libneo builds native macOS application windows and glass effects with GPUI and
//! AppKit.

pub mod glass;
mod lifecycle;
mod native_views;
mod platform;
pub mod theme;
pub mod window;

pub use lifecycle::{NativeRoot, install};
