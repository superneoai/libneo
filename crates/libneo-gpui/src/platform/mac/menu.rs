//! Connects GPUI menus to AppKit's standard application behavior.

use objc2::MainThreadMarker;
use objc2_app_kit::NSApplication;
use objc2_foundation::NSString;

/// Marks the Help menu so AppKit installs menu search in it.
pub(crate) fn configure_system_roles() {
    let mtm = MainThreadMarker::new().expect("menus must be installed on the main thread");
    let application = NSApplication::sharedApplication(mtm);
    let Some(menu_bar) = application.mainMenu() else {
        return;
    };
    let Some(help_item) = menu_bar.itemWithTitle(&NSString::from_str("Help")) else {
        return;
    };
    application.setHelpMenu(help_item.submenu().as_deref());
}

/// Opens AppKit's standard About panel.
pub(crate) fn show_about() {
    let mtm = MainThreadMarker::new().expect("the About panel must open on the main thread");
    NSApplication::sharedApplication(mtm).orderFrontStandardAboutPanel(None);
}

/// Brings all application windows to the front.
pub(crate) fn bring_all_to_front() {
    let mtm = MainThreadMarker::new().expect("windows must be ordered on the main thread");
    NSApplication::sharedApplication(mtm).arrangeInFront(None);
}
