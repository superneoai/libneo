//! Builds the application menu bar.
//!
//! Menu commands use GPUI actions. Register application-specific handlers in
//! GPUI as usual; the focused action handler determines whether a command is
//! available when its menu opens.

pub use gpui::{Action, App};

use gpui::KeyBinding;

/// A menu bar installed for the application.
#[derive(Default)]
pub struct MenuBar {
    menus: Vec<Menu>,
}

impl MenuBar {
    /// Creates an empty menu bar.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the top-level menus in their display order.
    pub fn menus(mut self, menus: impl IntoIterator<Item = Menu>) -> Self {
        self.menus = menus.into_iter().collect();
        self
    }

    /// Installs the menu bar, its shortcuts, and its macOS system roles.
    ///
    /// Call [`crate::install`] first and install the menu bar once. Shortcuts
    /// become GPUI key bindings and dispatch the same actions as their menu
    /// items. Use focused action-handler availability for dynamic state; GPUI
    /// cannot remove only the key bindings from an earlier installation.
    ///
    /// # Panics
    ///
    /// This function panics if libneo-gpui is not installed or if a shortcut
    /// is not valid GPUI keystroke syntax.
    pub fn install(self, cx: &mut App) {
        crate::lifecycle::assert_installed(cx);

        let mut shortcuts = Vec::new();
        let menus = self
            .menus
            .into_iter()
            .map(|menu| menu.into_gpui(&mut shortcuts))
            .collect::<Vec<_>>();
        cx.bind_keys(shortcuts);
        cx.set_menus(menus);
        crate::platform::menu::configure_system_roles();
    }
}

/// A named top-level menu or submenu.
pub struct Menu {
    name: String,
    items: Vec<MenuItem>,
    enabled: bool,
}

impl Menu {
    /// Creates an empty menu with the supplied title.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            items: Vec::new(),
            enabled: true,
        }
    }

    /// Creates the standard macOS application menu.
    ///
    /// The menu substitutes `application_name` into About, Hide, and Quit.
    /// Settings dispatches [`Settings`]; without an application handler GPUI
    /// presents it as unavailable. The remaining items use libneo's standard
    /// handlers, and Services is populated by macOS.
    pub fn application(application_name: impl Into<String>) -> Self {
        let application_name = application_name.into();
        Self::new(application_name.clone()).items([
            MenuItem::action(format!("About {application_name}"), About),
            MenuItem::separator(),
            MenuItem::action("Settings…", Settings).shortcut("cmd-,"),
            MenuItem::separator(),
            MenuItem::services(),
            MenuItem::separator(),
            MenuItem::action(format!("Hide {application_name}"), Hide).shortcut("cmd-h"),
            MenuItem::action("Hide Others", HideOthers).shortcut("cmd-alt-h"),
            MenuItem::action("Show All", ShowAll),
            MenuItem::separator(),
            MenuItem::action(format!("Quit {application_name}"), Quit).shortcut("cmd-q"),
        ])
    }

    /// Creates the standard macOS Window menu.
    ///
    /// macOS appends the application's open windows to this menu.
    pub fn window() -> Self {
        Self::new("Window").items([
            MenuItem::action("Minimize", Minimize).shortcut("cmd-m"),
            MenuItem::action("Zoom", Zoom),
            MenuItem::separator(),
            MenuItem::action("Bring All to Front", BringAllToFront),
        ])
    }

    /// Creates the standard macOS Help menu.
    ///
    /// AppKit adds its menu-search field when the menu bar is installed.
    pub fn help() -> Self {
        Self::new("Help")
    }

    /// Sets the items in this menu.
    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    /// Sets whether this menu can be selected.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    fn into_gpui(self, shortcuts: &mut Vec<KeyBinding>) -> gpui::Menu {
        gpui::Menu::new(self.name)
            .items(self.items.into_iter().map(|item| item.into_gpui(shortcuts)))
            .disabled(!self.enabled)
    }
}

/// An action, separator, submenu, or system-provided submenu in a menu.
pub struct MenuItem {
    kind: MenuItemKind,
}

enum MenuItemKind {
    Separator,
    Submenu(Menu),
    Services,
    Action {
        name: String,
        action: Box<dyn Action>,
        shortcut: Option<String>,
        shortcut_factory: ShortcutFactory,
        enabled: bool,
    },
}

type ShortcutFactory = Box<dyn Fn(&str) -> KeyBinding>;

impl MenuItem {
    /// Creates an item that dispatches a GPUI action.
    ///
    /// GPUI enables the item when the action has a handler in the focused
    /// dispatch path or a global handler.
    pub fn action<A>(name: impl Into<String>, action: A) -> Self
    where
        A: Action + Clone + 'static,
    {
        let shortcut_action = action.clone();
        Self {
            kind: MenuItemKind::Action {
                name: name.into(),
                action: Box::new(action),
                shortcut: None,
                shortcut_factory: Box::new(move |keys| {
                    KeyBinding::new(keys, shortcut_action.clone(), None)
                }),
                enabled: true,
            },
        }
    }

    /// Creates a visual separator between groups of items.
    pub fn separator() -> Self {
        Self {
            kind: MenuItemKind::Separator,
        }
    }

    /// Creates a nested menu.
    pub fn submenu(menu: Menu) -> Self {
        Self {
            kind: MenuItemKind::Submenu(menu),
        }
    }

    /// Creates the Services submenu populated and managed by macOS.
    pub fn services() -> Self {
        Self {
            kind: MenuItemKind::Services,
        }
    }

    /// Assigns a keyboard shortcut using GPUI keystroke syntax.
    ///
    /// This has no effect on separators or system-provided submenus.
    pub fn shortcut(mut self, keys: impl Into<String>) -> Self {
        if let MenuItemKind::Action { shortcut, .. } = &mut self.kind {
            *shortcut = Some(keys.into());
        }
        self
    }

    /// Sets whether an action or submenu is available.
    ///
    /// Separators and system-provided submenus ignore this setting. GPUI also
    /// validates actions against the focused dispatch path when a menu opens.
    pub fn enabled(mut self, enabled: bool) -> Self {
        match &mut self.kind {
            MenuItemKind::Action {
                enabled: current, ..
            } => *current = enabled,
            MenuItemKind::Submenu(menu) => menu.enabled = enabled,
            MenuItemKind::Separator | MenuItemKind::Services => {}
        }
        self
    }

    fn into_gpui(self, shortcuts: &mut Vec<KeyBinding>) -> gpui::MenuItem {
        match self.kind {
            MenuItemKind::Separator => gpui::MenuItem::separator(),
            MenuItemKind::Submenu(menu) => gpui::MenuItem::submenu(menu.into_gpui(shortcuts)),
            MenuItemKind::Services => {
                gpui::MenuItem::os_submenu("Services", gpui::SystemMenuType::Services)
            }
            MenuItemKind::Action {
                name,
                action,
                shortcut,
                shortcut_factory,
                enabled,
            } => {
                if enabled {
                    if let Some(keys) = shortcut {
                        shortcuts.push(shortcut_factory(&keys));
                    }
                    gpui::MenuItem::Action {
                        name: name.into(),
                        action,
                        os_action: None,
                        checked: false,
                        disabled: false,
                    }
                } else {
                    gpui::MenuItem::action(name, gpui::NoAction).disabled(true)
                }
            }
        }
    }
}

gpui::actions!(
    libneo,
    [
        /// Opens the standard application About panel.
        About,
        /// Opens the application's settings interface.
        Settings,
        /// Hides the application.
        Hide,
        /// Hides every other application.
        HideOthers,
        /// Shows all hidden applications.
        ShowAll,
        /// Quits the application.
        Quit,
        /// Minimizes the active window.
        Minimize,
        /// Zooms the active window.
        Zoom,
        /// Brings every application window to the front.
        BringAllToFront,
    ]
);

pub(crate) fn init(cx: &mut App) {
    cx.on_action(|_: &About, _| crate::platform::menu::show_about())
        .on_action(|_: &Hide, cx| cx.hide())
        .on_action(|_: &HideOthers, cx| cx.hide_other_apps())
        .on_action(|_: &ShowAll, cx| cx.unhide_other_apps())
        .on_action(|_: &Quit, cx| cx.quit())
        .on_action(|_: &Minimize, cx| {
            cx.defer(|cx| {
                if let Some(handle) = cx.active_window() {
                    let _ = handle.update(cx, |_, window, _| window.minimize_window());
                }
            });
        })
        .on_action(|_: &Zoom, cx| {
            cx.defer(|cx| {
                if let Some(handle) = cx.active_window() {
                    let _ = handle.update(cx, |_, window, _| window.zoom_window());
                }
            });
        })
        .on_action(|_: &BringAllToFront, _| {
            crate::platform::menu::bring_all_to_front();
        });
}

#[cfg(test)]
mod tests {
    use super::{Menu, MenuItem};

    #[test]
    fn application_menu_substitutes_the_application_name() {
        let menu = Menu::application("Example");
        let mut shortcuts = Vec::new();
        let menu = menu.into_gpui(&mut shortcuts).owned();

        assert_eq!(menu.name.as_ref(), "Example");
        assert_eq!(menu.items.len(), 11);
        assert_eq!(shortcuts.len(), 4);
        assert!(matches!(menu.items[4], gpui::OwnedMenuItem::SystemMenu(_)));
        assert!(
            matches!(menu.items[10], gpui::OwnedMenuItem::Action { ref name, .. } if name == "Quit Example")
        );
    }

    #[test]
    fn nested_shortcuts_and_disabled_state_are_preserved() {
        let menu = Menu::new("Tools").items([
            MenuItem::action("Unavailable", super::Settings).enabled(false),
            MenuItem::submenu(
                Menu::new("Mode")
                    .items([MenuItem::action("Choose", super::Settings).shortcut("cmd-shift-m")]),
            ),
        ]);
        let mut shortcuts = Vec::new();
        let menu = menu.into_gpui(&mut shortcuts).owned();

        assert_eq!(shortcuts.len(), 1);
        assert!(matches!(
            menu.items[0],
            gpui::OwnedMenuItem::Action { disabled: true, .. }
        ));
        assert!(matches!(menu.items[1], gpui::OwnedMenuItem::Submenu(_)));
    }
}
