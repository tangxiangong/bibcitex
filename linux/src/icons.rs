//! Reuse the native clients' standalone SVG artwork.
use gpui_kit::component::Icon;

pub fn icon(name: &str) -> Icon {
    Icon::default().path(format!("bibcitex-icons/{name}.svg"))
}

pub fn load(path: &str) -> Option<&'static [u8]> {
    macro_rules! assets {
        ($($name:literal),* $(,)?) => {
            match path {
                $(concat!("bibcitex-icons/", $name, ".svg") =>
                    Some(include_bytes!(concat!("../../public/icons/", $name, ".svg")).as_slice()),)*
                _ => None,
            }
        };
    }
    assets!(
        "search",
        "library",
        "fileText",
        "chevronDown",
        "chevronRight",
        "check",
        "folderOpen",
        "more",
        "add",
        "panelLeftOpen",
        "panelLeftClose",
        "panelRightOpen",
        "panelRightClose",
        "refresh",
        "externalLink",
        "x",
        "settings",
        "info",
        "pin",
        "alert",
        "copy",
        "clipboard",
        "link",
        "folderAdd",
        "rename",
    )
}
