//! Implements the native views with AppKit.

pub(crate) mod chrome;
pub(crate) mod glass;
mod handle;
pub(crate) mod table;
pub(crate) mod visual_effect;

use objc2::MainThreadMarker;
use objc2_foundation::{NSOperatingSystemVersion, NSProcessInfo};

const MINIMUM_MACOS: NSOperatingSystemVersion = NSOperatingSystemVersion {
    majorVersion: 26,
    minorVersion: 1,
    patchVersion: 0,
};

/// Returns the main-thread marker for a supported system.
///
/// # Panics
///
/// This function panics off the main thread, and on systems before macOS 26.1.
pub(crate) fn assert_supported_os() -> MainThreadMarker {
    let mtm = MainThreadMarker::new().expect("libneo must be installed on the main thread");
    let process_info = NSProcessInfo::processInfo();
    assert!(
        process_info.isOperatingSystemAtLeastVersion(MINIMUM_MACOS),
        "the system must be macOS 26.1 or later"
    );
    mtm
}
