//! Builds native window toolbars.
//!
//! Toolbar commands use GPUI actions. Register application-specific handlers in
//! GPUI as usual; the focused action handler determines whether a command is
//! available.

pub use gpui::Action;

/// Selects how AppKit displays toolbar items.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolbarDisplayMode {
    /// Uses the display mode selected by the system or user.
    System,
    /// Shows each item's icon and label.
    IconAndLabel,
    /// Shows only item icons.
    IconOnly,
    /// Shows only item labels.
    LabelOnly,
}

/// Selects the AppKit toolbar style.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolbarStyle {
    /// Lets AppKit choose the toolbar style.
    Automatic,
    /// Uses the expanded toolbar style.
    Expanded,
    /// Uses the preferences-window toolbar style.
    Preference,
    /// Uses the unified toolbar style.
    Unified,
    /// Uses the compact unified toolbar style.
    UnifiedCompact,
}

/// Configures native toolbar presentation and persistence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToolbarConfiguration {
    /// Controls how toolbar item icons and labels appear.
    pub display_mode: ToolbarDisplayMode,
    /// Controls the toolbar's AppKit style.
    pub style: ToolbarStyle,
    /// Controls whether AppKit persists toolbar changes under its identifier.
    pub autosaves_configuration: bool,
    /// Controls whether the user can customize toolbar items.
    pub allows_user_customization: bool,
}

/// A native toolbar installed in one window.
#[derive(Clone, Debug)]
pub struct Toolbar {
    pub(crate) identifier: String,
    pub(crate) configuration: ToolbarConfiguration,
    pub(crate) items: Vec<ToolbarItem>,
}

impl Toolbar {
    /// Creates an empty toolbar with the supplied AppKit identifier and
    /// presentation configuration.
    ///
    /// Use a stable identifier that is unique to this kind of window.
    pub fn new(identifier: impl Into<String>, configuration: ToolbarConfiguration) -> Self {
        Self {
            identifier: identifier.into(),
            configuration,
            items: Vec::new(),
        }
    }

    /// Sets the toolbar items in their display order.
    pub fn items(mut self, items: impl IntoIterator<Item = ToolbarItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }
}

/// An action or system-provided item in a toolbar.
pub struct ToolbarItem {
    pub(crate) kind: ToolbarItemKind,
}

pub(crate) enum ToolbarItemKind {
    Action {
        identifier: String,
        label: String,
        symbol: Option<String>,
        action: Box<dyn Action>,
        enabled: bool,
    },
    System(ToolbarSystemItem),
}

impl Clone for ToolbarItem {
    fn clone(&self) -> Self {
        let kind = match &self.kind {
            ToolbarItemKind::Action {
                identifier,
                label,
                symbol,
                action,
                enabled,
            } => ToolbarItemKind::Action {
                identifier: identifier.clone(),
                label: label.clone(),
                symbol: symbol.clone(),
                action: action.boxed_clone(),
                enabled: *enabled,
            },
            ToolbarItemKind::System(item) => ToolbarItemKind::System(*item),
        };
        Self { kind }
    }
}

impl std::fmt::Debug for ToolbarItem {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ToolbarItemKind::Action {
                identifier,
                label,
                symbol,
                enabled,
                ..
            } => formatter
                .debug_struct("ToolbarItem")
                .field("identifier", identifier)
                .field("label", label)
                .field("symbol", symbol)
                .field("enabled", enabled)
                .finish(),
            ToolbarItemKind::System(item) => {
                formatter.debug_tuple("ToolbarItem").field(item).finish()
            }
        }
    }
}

impl ToolbarItem {
    /// Creates a labeled item that dispatches a GPUI action.
    ///
    /// GPUI enables the item when the action has a handler in the focused
    /// dispatch path or a global handler. Identifiers must be unique among the
    /// non-system items in one toolbar.
    pub fn action<A>(identifier: impl Into<String>, label: impl Into<String>, action: A) -> Self
    where
        A: Action + Clone + 'static,
    {
        Self {
            kind: ToolbarItemKind::Action {
                identifier: identifier.into(),
                label: label.into(),
                symbol: None,
                action: Box::new(action),
                enabled: true,
            },
        }
    }

    /// Shows an SF Symbol on an action item.
    ///
    /// This has no effect on system-provided items. The label remains available
    /// to accessibility and the toolbar overflow menu.
    pub fn symbol(mut self, symbol: impl Into<String>) -> Self {
        if let ToolbarItemKind::Action {
            symbol: current, ..
        } = &mut self.kind
        {
            *current = Some(symbol.into());
        }
        self
    }

    /// Creates an item managed by AppKit.
    pub fn system(item: ToolbarSystemItem) -> Self {
        Self {
            kind: ToolbarItemKind::System(item),
        }
    }

    /// Sets whether an action item is available.
    ///
    /// System-provided items ignore this setting. GPUI also validates actions
    /// against the focused dispatch path.
    pub fn enabled(mut self, enabled: bool) -> Self {
        if let ToolbarItemKind::Action {
            enabled: current, ..
        } = &mut self.kind
        {
            *current = enabled;
        }
        self
    }
}

/// Selects an AppKit-provided toolbar item.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolbarSystemItem {
    /// Expands to consume the available space between neighboring items.
    FlexibleSpace,
    /// Inserts a fixed-width space.
    Space,
    /// Toggles the window's sidebar.
    ToggleSidebar,
    /// Toggles the window's inspector.
    ToggleInspector,
    /// Opens the standard print workflow.
    Print,
}

#[cfg(test)]
mod tests {
    use super::{
        Toolbar, ToolbarConfiguration, ToolbarDisplayMode, ToolbarItem, ToolbarItemKind,
        ToolbarStyle, ToolbarSystemItem,
    };

    fn configuration() -> ToolbarConfiguration {
        ToolbarConfiguration {
            display_mode: ToolbarDisplayMode::IconAndLabel,
            style: ToolbarStyle::Unified,
            autosaves_configuration: false,
            allows_user_customization: false,
        }
    }

    #[test]
    fn toolbar_can_be_empty() {
        let toolbar = Toolbar::new("example.toolbar", configuration());

        assert!(toolbar.items.is_empty());
    }

    #[test]
    fn action_state_and_system_items_are_preserved() {
        let toolbar = Toolbar::new("example.toolbar", configuration()).items([
            ToolbarItem::action("example.search", "Search", gpui::NoAction)
                .symbol("magnifyingglass")
                .enabled(false),
            ToolbarItem::system(ToolbarSystemItem::FlexibleSpace),
        ]);

        assert!(matches!(
            toolbar.items[0].kind,
            ToolbarItemKind::Action {
                enabled: false,
                ref symbol,
                ..
            } if symbol.as_deref() == Some("magnifyingglass")
        ));
        assert!(matches!(
            toolbar.items[1].kind,
            ToolbarItemKind::System(ToolbarSystemItem::FlexibleSpace)
        ));
    }
}
