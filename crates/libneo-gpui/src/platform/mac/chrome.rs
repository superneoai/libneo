//! Builds the native window chrome.

use gpui::{AsyncApp, Window};
use objc2::rc::Retained;
use objc2::runtime::{Bool, ProtocolObject};
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSAutoresizingMaskOptions, NSColor, NSImage, NSSplitViewController, NSSplitViewItem, NSToolbar,
    NSToolbarDelegate, NSToolbarDisplayMode, NSToolbarFlexibleSpaceItemIdentifier, NSToolbarItem,
    NSToolbarItemIdentifier, NSToolbarItemValidation, NSToolbarPrintItemIdentifier,
    NSToolbarSpaceItemIdentifier, NSToolbarToggleInspectorItemIdentifier,
    NSToolbarToggleSidebarItemIdentifier, NSView, NSViewController, NSWindowStyleMask,
    NSWindowToolbarStyle,
};
use objc2_foundation::{NSArray, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString};

use crate::toolbar::{
    Toolbar, ToolbarDisplayMode, ToolbarItemKind, ToolbarStyle, ToolbarSystemItem,
};
use crate::window::WindowChrome;

use super::handle::NativeWindowHandle;

pub(crate) struct ToolbarDelegateIvars {
    toolbar: Toolbar,
    cx: AsyncApp,
}

impl std::fmt::Debug for ToolbarDelegateIvars {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ToolbarDelegateIvars")
            .field("toolbar", &self.toolbar)
            .finish_non_exhaustive()
    }
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "LNGpuiToolbarDelegate"]
    #[ivars = ToolbarDelegateIvars]
    #[derive(Debug)]
    pub(crate) struct ToolbarDelegate;

    unsafe impl NSObjectProtocol for ToolbarDelegate {}

    unsafe impl NSToolbarDelegate for ToolbarDelegate {
        #[unsafe(method_id(toolbarDefaultItemIdentifiers:))]
        fn default_identifiers(&self, _toolbar: &NSToolbar) -> Retained<NSArray<NSString>> {
            identifiers(&self.ivars().toolbar)
        }

        #[unsafe(method_id(toolbarAllowedItemIdentifiers:))]
        fn allowed_identifiers(&self, _toolbar: &NSToolbar) -> Retained<NSArray<NSString>> {
            identifiers(&self.ivars().toolbar)
        }

        #[unsafe(method_id(toolbar:itemForItemIdentifier:willBeInsertedIntoToolbar:))]
        fn item_for_identifier(
            &self,
            _toolbar: &NSToolbar,
            identifier: &NSToolbarItemIdentifier,
            _inserted: bool,
        ) -> Option<Retained<NSToolbarItem>> {
            make_item(self, identifier, MainThreadMarker::from(self))
        }
    }

    unsafe impl NSToolbarItemValidation for ToolbarDelegate {
        #[unsafe(method(validateToolbarItem:))]
        fn validate_toolbar_item(&self, item: &NSToolbarItem) -> Bool {
            let identifier = item.itemIdentifier().to_string();
            let Some((enabled, action)) = action_for_identifier(&self.ivars().toolbar, &identifier)
            else {
                return true.into();
            };
            (enabled && self.ivars().cx.update(|cx| cx.is_action_available(action))).into()
        }
    }

    impl ToolbarDelegate {
        #[unsafe(method(performToolbarAction:))]
        fn perform_toolbar_action(&self, sender: &NSToolbarItem) {
            let identifier = sender.itemIdentifier().to_string();
            let Some((true, action)) = action_for_identifier(&self.ivars().toolbar, &identifier)
            else {
                return;
            };
            let action = action.boxed_clone();
            self.ivars().cx.update(|cx| cx.dispatch_action(&*action));
        }
    }
);

impl ToolbarDelegate {
    fn new(toolbar: Toolbar, cx: AsyncApp, mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(ToolbarDelegateIvars { toolbar, cx });
        // SAFETY: the ivars are initialized before NSObject initialization.
        unsafe { msg_send![super(this), init] }
    }
}

fn identifiers(toolbar: &Toolbar) -> Retained<NSArray<NSString>> {
    let names = toolbar
        .items
        .iter()
        .map(|item| match &item.kind {
            ToolbarItemKind::Action { identifier, .. } => identifier.clone(),
            ToolbarItemKind::System(item) => system_identifier(*item).to_string(),
        })
        .map(|identifier| NSString::from_str(&identifier))
        .collect::<Vec<_>>();
    let names = names.iter().map(|name| &**name).collect::<Vec<_>>();
    NSArray::from_slice(&names)
}

fn system_identifier(item: ToolbarSystemItem) -> &'static NSToolbarItemIdentifier {
    // SAFETY: AppKit defines these immutable process-lifetime identifiers.
    unsafe {
        match item {
            ToolbarSystemItem::FlexibleSpace => NSToolbarFlexibleSpaceItemIdentifier,
            ToolbarSystemItem::Space => NSToolbarSpaceItemIdentifier,
            ToolbarSystemItem::ToggleSidebar => NSToolbarToggleSidebarItemIdentifier,
            ToolbarSystemItem::ToggleInspector => NSToolbarToggleInspectorItemIdentifier,
            ToolbarSystemItem::Print => NSToolbarPrintItemIdentifier,
        }
    }
}

fn action_for_identifier<'a>(
    toolbar: &'a Toolbar,
    identifier: &str,
) -> Option<(bool, &'a dyn gpui::Action)> {
    toolbar.items.iter().find_map(|item| match &item.kind {
        ToolbarItemKind::Action {
            identifier: current,
            action,
            enabled,
            ..
        } if current == identifier => Some((*enabled, &**action)),
        ToolbarItemKind::Action { .. } | ToolbarItemKind::System(_) => None,
    })
}

fn make_item(
    delegate: &ToolbarDelegate,
    identifier: &NSToolbarItemIdentifier,
    mtm: MainThreadMarker,
) -> Option<Retained<NSToolbarItem>> {
    let identifier_text = identifier.to_string();
    let item_definition =
        delegate
            .ivars()
            .toolbar
            .items
            .iter()
            .find_map(|item| match &item.kind {
                ToolbarItemKind::Action {
                    identifier,
                    label,
                    symbol,
                    enabled,
                    ..
                } if identifier == &identifier_text => Some((label, symbol.as_deref(), *enabled)),
                ToolbarItemKind::Action { .. } | ToolbarItemKind::System(_) => None,
            })?;

    let item = NSToolbarItem::initWithItemIdentifier(NSToolbarItem::alloc(mtm), identifier);
    let label = NSString::from_str(item_definition.0);
    item.setLabel(&label);
    item.setPaletteLabel(&label);
    item.setToolTip(Some(&label));
    if let Some(symbol) = item_definition.1
        && let Some(image) = NSImage::imageWithSystemSymbolName_accessibilityDescription(
            &NSString::from_str(symbol),
            Some(&label),
        )
    {
        item.setImage(Some(&image));
    }
    item.setEnabled(item_definition.2);
    // SAFETY: the delegate implements the selector and outlives every item.
    unsafe {
        item.setTarget(Some(delegate));
        item.setAction(Some(sel!(performToolbarAction:)));
    }
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
    cx: AsyncApp,
) -> Result<Option<NativeWindowChrome>, String> {
    let WindowChrome::Toolbar(toolbar_configuration) = chrome else {
        return Ok(None);
    };
    let handle = NativeWindowHandle::acquire(gpui_window, mtm)?;
    let window = handle.window();

    // The style mask applies before the content view controller, so that the
    // content fills the window.
    window.setStyleMask(window.styleMask() | NSWindowStyleMask::FullSizeContentView);
    host_content(&handle, mtm);

    let identifier = NSString::from_str(&toolbar_configuration.identifier);
    let configuration = toolbar_configuration.configuration;
    let delegate = ToolbarDelegate::new(toolbar_configuration, cx, mtm);
    let toolbar = NSToolbar::initWithIdentifier(NSToolbar::alloc(mtm), &identifier);
    toolbar.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
    toolbar.setDisplayMode(match configuration.display_mode {
        ToolbarDisplayMode::System => NSToolbarDisplayMode::Default,
        ToolbarDisplayMode::IconAndLabel => NSToolbarDisplayMode::IconAndLabel,
        ToolbarDisplayMode::IconOnly => NSToolbarDisplayMode::IconOnly,
        ToolbarDisplayMode::LabelOnly => NSToolbarDisplayMode::LabelOnly,
    });
    toolbar.setAutosavesConfiguration(configuration.autosaves_configuration);
    toolbar.setAllowsUserCustomization(configuration.allows_user_customization);

    window.setToolbar(Some(&toolbar));
    window.setToolbarStyle(match configuration.style {
        ToolbarStyle::Automatic => NSWindowToolbarStyle::Automatic,
        ToolbarStyle::Expanded => NSWindowToolbarStyle::Expanded,
        ToolbarStyle::Preference => NSWindowToolbarStyle::Preference,
        ToolbarStyle::Unified => NSWindowToolbarStyle::Unified,
        ToolbarStyle::UnifiedCompact => NSWindowToolbarStyle::UnifiedCompact,
    });
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
