//! Native services shared by the three NSH desktop hosts.
mod bridge;
#[cfg(target_os = "linux")]
pub mod desktop;
pub mod files;
pub mod instance;
#[cfg(test)]
mod locale_build;
mod locale_key;
mod math;
mod platform;
mod preferences;
pub mod shortcut;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Command {
    Main,
    Helper,
    HelperToken(String),
    MainToken(String),
    Tray,
    #[serde(skip)]
    TrayAt {
        x: i32,
        y: i32,
    },
    #[serde(skip)]
    CheckUpdates,
    Settings,
    #[serde(skip)]
    About,
    Quit,
    #[serde(skip)]
    RefreshLibraries,
    #[serde(skip)]
    Error(String),
    #[serde(skip)]
    TrayAvailable(bool),
    #[serde(skip)]
    ShortcutChanged,
    #[serde(skip)]
    PasteFailed {
        error: String,
        key: String,
    },
}

/// Registers the desktop bibliography library, search, preferences and system interfaces.
#[nsh_core::prelude::native_module("bibcitex/native")]
fn native(registry: &mut nsh_core::element::Registry) {
    registry
        .module::<bridge::catalog::Catalog>()
        .module::<bridge::query::Query>()
        .module::<bridge::tags::Tags>()
        .module::<bridge::support::Files>()
        .module::<math::MathRender>()
        .module::<bridge::support::Shortcut>()
        .class::<bridge::search::SearchSession>()
        .class::<bridge::desktop::DesktopSession>();
}

#[cfg(test)]
mod registry_tests {
    #[test]
    fn host_can_register_both_sets_and_every_native_export_has_docs() {
        let native = super::registry();
        assert_eq!(native.specifier(), Some("bibcitex/native"));
        assert!(
            native.undocumented().is_empty(),
            "{:?}",
            native.undocumented()
        );
        let mut modules: Vec<_> = native.module_specs().iter().map(|spec| spec.name).collect();
        modules.sort_unstable();
        assert_eq!(
            modules,
            [
                "Catalog",
                "Files",
                "MathRender",
                "Query",
                "Shortcut",
                "Tags"
            ]
        );
        let mut classes: Vec<_> = native.class_specs().iter().map(|spec| spec.name).collect();
        classes.sort_unstable();
        assert_eq!(classes, ["DesktopSession", "SearchSession"]);
        // The release hosts merge both sets; duplicate native type names panic at startup.
        nsh_core::element::Registry::merged(&[nsh_base::registry(), native]);
    }
}
