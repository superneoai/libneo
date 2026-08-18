//! Builds the native window chrome.

use gpui::Window;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{
    NSAutoresizingMaskOptions, NSColor, NSImage, NSSplitViewController, NSSplitViewItem, NSToolbar,
    NSToolbarDelegate, NSToolbarDisplayMode, NSToolbarItem, NSToolbarItemIdentifier, NSView,
    NSViewController, NSWindowStyleMask, NSWindowToolbarStyle,
};
use objc2_foundation::{NSArray, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString};

use crate::window::WindowChrome;

use super::handle::NativeWindowHandle;

/// Pairs each toolbar item identifier with its symbol name.
const ITEM_SYMBOLS: [(&str, &str); 2] = [
    ("neo.sidebar", "sidebar.leading"),
    ("neo.search", "magnifyingglass"),
];

/// Identifies the flexible space between the toolbar items.
const FLEXIBLE_SPACE: &str = "NSToolbarFlexibleSpaceItem";

/// Identifies the toolbar of the application window.
const TOOLBAR_IDENTIFIER: &str = "neo.toolbar";

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "NeoToolbarDelegate"]
    #[derive(Debug)]
    pub(crate) struct ToolbarDelegate;

    unsafe impl NSObjectProtocol for ToolbarDelegate {}

    unsafe impl NSToolbarDelegate for ToolbarDelegate {
        #[unsafe(method_id(toolbarDefaultItemIdentifiers:))]
        fn default_identifiers(&self, _toolbar: &NSToolbar) -> Retained<NSArray<NSString>> {
            identifiers()
        }

        #[unsafe(method_id(toolbarAllowedItemIdentifiers:))]
        fn allowed_identifiers(&self, _toolbar: &NSToolbar) -> Retained<NSArray<NSString>> {
            identifiers()
        }

        #[unsafe(method_id(toolbar:itemForItemIdentifier:willBeInsertedIntoToolbar:))]
        fn item_for_identifier(
            &self,
            _toolbar: &NSToolbar,
            identifier: &NSToolbarItemIdentifier,
            _inserted: bool,
        ) -> Option<Retained<NSToolbarItem>> {
            make_item(identifier, MainThreadMarker::from(self))
        }
    }
);

fn identifiers() -> Retained<NSArray<NSString>> {
    let names: Vec<Retained<NSString>> = vec![
        NSString::from_str(ITEM_SYMBOLS[0].0),
        NSString::from_str(FLEXIBLE_SPACE),
        NSString::from_str(ITEM_SYMBOLS[1].0),
    ];
    let refs: Vec<&NSString> = names.iter().map(|name| &**name).collect();
    NSArray::from_slice(&refs)
}

fn make_item(
    identifier: &NSToolbarItemIdentifier,
    mtm: MainThreadMarker,
) -> Option<Retained<NSToolbarItem>> {
    let name = identifier.to_string();
    let symbol = ITEM_SYMBOLS
        .iter()
        .find(|(item, _)| *item == name)
        .map(|(_, symbol)| *symbol)?;
    let item = NSToolbarItem::initWithItemIdentifier(NSToolbarItem::alloc(mtm), identifier);
    let image = NSImage::imageWithSystemSymbolName_accessibilityDescription(
        &NSString::from_str(symbol),
        None,
    )?;
    item.setImage(Some(&image));
    item.setLabel(&NSString::from_str(symbol));
    Some(item)
}

/// Owns the chrome of one window.
pub(crate) struct NativeWindowChrome {
    _delegate: Retained<ToolbarDelegate>,
    toolbar: Retained<NSToolbar>,
}

/// Applies the chrome to a window.
pub(crate) fn configure(
    gpui_window: &Window,
    chrome: WindowChrome,
    mtm: MainThreadMarker,
) -> Result<Option<NativeWindowChrome>, String> {
    let WindowChrome::Toolbar = chrome else {
        return Ok(None);
    };
    let handle = NativeWindowHandle::acquire(gpui_window, mtm)?;
    let window = handle.window();

    // The style mask applies before the content view controller, so that the
    // content fills the window.
    window.setStyleMask(window.styleMask() | NSWindowStyleMask::FullSizeContentView);
    host_content(&handle, mtm);

    // SAFETY: the delegate is allocated on the main thread, has no ivars, and
    // uses NSObject's designated initializer.
    let delegate: Retained<ToolbarDelegate> =
        unsafe { msg_send![ToolbarDelegate::alloc(mtm), init] };
    let toolbar = NSToolbar::initWithIdentifier(
        NSToolbar::alloc(mtm),
        &NSString::from_str(TOOLBAR_IDENTIFIER),
    );
    toolbar.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
    toolbar.setDisplayMode(NSToolbarDisplayMode::IconOnly);
    toolbar.setAutosavesConfiguration(false);
    toolbar.setAllowsUserCustomization(false);

    window.setToolbar(Some(&toolbar));
    window.setToolbarStyle(NSWindowToolbarStyle::Unified);
    // The system color lets the toolbar sample the window content.
    let background = NSColor::windowBackgroundColor();
    window.setBackgroundColor(Some(&background));

    Ok(Some(NativeWindowChrome {
        _delegate: delegate,
        toolbar,
    }))
}

/// Puts the GPUI view inside a split view controller.
///
/// The split item supplies the safe area of the content.
fn host_content(handle: &NativeWindowHandle, mtm: MainThreadMarker) {
    let window = handle.window();
    let bounds = window.contentView().map_or_else(
        || NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0)),
        |view| view.bounds(),
    );

    let content_controller = NSViewController::new(mtm);
    let host = NSView::initWithFrame(NSView::alloc(mtm), bounds);
    content_controller.setView(&host);

    let item = NSSplitViewItem::splitViewItemWithViewController(&content_controller);
    item.setAutomaticallyAdjustsSafeAreaInsets(true);

    let split = NSSplitViewController::new(mtm);
    split.addSplitViewItem(&item);

    let gpui_view = handle.retained_gpui_view();
    gpui_view.removeFromSuperview();
    window.setContentViewController(Some(&split));
    host.addSubview(&gpui_view);
    gpui_view.setFrame(host.bounds());
    gpui_view.setAutoresizingMask(
        NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
    );
}

impl Drop for NativeWindowChrome {
    fn drop(&mut self) {
        debug_assert!(
            MainThreadMarker::new().is_some(),
            "the chrome releases on the main thread"
        );
        self.toolbar.setDelegate(None);
    }
}
