//! Selects the platform implementation.

#[cfg(not(target_os = "macos"))]
compile_error!("libneo-gpui runs on macOS 26.1 and later");

#[cfg(target_os = "macos")]
pub(crate) mod mac;

#[cfg(target_os = "macos")]
pub(crate) use mac::color::{
    reduce_motion_enabled, reduce_transparency_enabled, resolve_system_color,
};
#[cfg(target_os = "macos")]
pub(crate) use mac::{assert_supported_os, chrome, menu};
