#![deny(clippy::undocumented_unsafe_blocks)]

//! libneo supplies application lifecycle and themes for GPUI on macOS.

mod lifecycle;
mod platform;
pub mod theme;

pub use lifecycle::{NativeRoot, install};
