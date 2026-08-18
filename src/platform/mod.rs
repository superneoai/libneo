//! Selects the platform implementation.

#[cfg(not(target_os = "macos"))]
compile_error!("libneo runs on macOS 26.1 and later");

#[cfg(target_os = "macos")]
pub(crate) mod mac;

#[cfg(target_os = "macos")]
pub(crate) use mac::{assert_supported_os, chrome};
