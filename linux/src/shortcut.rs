//! Normalize shortcut preferences in the notation used by the native desktop clients.
#[cfg(target_os = "macos")]
pub const DEFAULT_SHORTCUT: &str = "Cmd+Shift+K";
#[cfg(target_os = "windows")]
pub const DEFAULT_SHORTCUT: &str = "Win+Shift+K";
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub const DEFAULT_SHORTCUT: &str = "Ctrl+Shift+K";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shortcut {
    pub modifiers: u16,
    pub keysym: u32,
    pub value: String,
}

fn meta_label() -> &'static str {
    match std::env::consts::OS {
        "macos" => "Cmd",
        "windows" => "Win",
        _ => "Super",
    }
}

impl Shortcut {
    pub fn display_value(&self) -> String {
        if cfg!(target_os = "windows") {
            return self.value.replace("+", " + ");
        }
        if cfg!(target_os = "macos") {
            let labels: String = [(4, "⌃"), (8, "⌥"), (1, "⇧"), (64, "⌘")]
                .into_iter()
                .filter(|(bit, _)| self.modifiers & bit != 0)
                .map(|(_, label)| label)
                .collect();
            let key = self.value.rsplit('+').next().unwrap_or_default();
            return labels + if key == "Space" { "␣" } else { key };
        }
        self.value
            .replace("Super+", "⊞")
            .replace("Ctrl+", "⌃")
            .replace("Alt+", "⌥")
            .replace("Shift+", "⇧")
            .replace("Space", "␣")
    }

    pub fn from_key(key: &str, modifiers: u16) -> Result<Self, String> {
        let mut names: Vec<_> = [(64, meta_label()), (4, "Ctrl"), (8, "Alt"), (1, "Shift")]
            .into_iter()
            .filter(|(bit, _)| modifiers & bit != 0)
            .map(|(_, name)| name)
            .collect();
        names.push(if key == " " || key == "space" {
            "Space"
        } else {
            key
        });
        Self::parse(&names.join("+"))
    }

    pub fn parse(text: &str) -> Result<Self, String> {
        let invalid = || {
            match std::env::consts::OS {
                "macos" => "error.helperShortcutFormatMac",
                "windows" => "error.helperShortcutFormatWindows",
                _ => "error.helperShortcutFormatLinux",
            }
            .to_owned()
        };
        let upper = text.to_ascii_uppercase();
        let mut parts: Vec<_> = upper.split('+').map(str::trim).collect();
        let key = parts.pop().ok_or_else(invalid)?;
        if parts.is_empty() {
            return Err(invalid());
        }
        let mut modifiers = 0;
        for part in parts {
            let bit = match part {
                "CTRL" | "CONTROL" => 4,
                "ALT" => 8,
                "OPT" | "OPTION" if cfg!(target_os = "macos") => 8,
                "CMD" | "COMMAND" if cfg!(target_os = "macos") => 64,
                "SHIFT" => 1,
                "SUPER" | "WIN" if !cfg!(target_os = "macos") => 64,
                "META" if cfg!(target_os = "linux") => 64,
                _ => return Err(invalid()),
            };
            if modifiers & bit != 0 {
                return Err(invalid());
            }
            modifiers |= bit;
        }
        if modifiers & (4 | 8 | 64) == 0 {
            return Err(invalid());
        }
        let keysym = match key {
            "SPACE" => 32,
            key if key.len() == 1 && key.as_bytes()[0].is_ascii_alphanumeric() => {
                u32::from(key.as_bytes()[0].to_ascii_lowercase())
            }
            key => key
                .strip_prefix('F')
                .and_then(|n| n.parse::<u32>().ok())
                .filter(|n| (1..=12).contains(n))
                .map(|n| 0xffbd + n)
                .ok_or_else(invalid)?,
        };
        let mut names: Vec<_> = [(64, meta_label()), (4, "Ctrl"), (8, "Alt"), (1, "Shift")]
            .into_iter()
            .filter(|(bit, _)| modifiers & bit != 0)
            .map(|(_, name)| name.to_owned())
            .collect();
        names.push(if key == "SPACE" {
            "Space".into()
        } else if keysym >= 0xffbe {
            format!("F{}", keysym - 0xffbd)
        } else {
            key.to_owned()
        });
        Ok(Self {
            modifiers,
            keysym,
            value: names.join("+"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcut_normalizes_aliases_and_modifier_order() {
        assert_eq!(
            Shortcut::parse(" shift + control + k ").unwrap(),
            Shortcut {
                modifiers: 5,
                keysym: u32::from(b'k'),
                value: "Ctrl+Shift+K".into(),
            }
        );
    }

    #[test]
    fn shortcut_accepts_space_digits_and_function_keys() {
        assert_eq!(Shortcut::parse("Alt+Space").unwrap().keysym, 32);
        assert_eq!(
            Shortcut::parse(&format!("{}+9", meta_label()))
                .unwrap()
                .value,
            format!("{}+9", meta_label())
        );
        assert_eq!(Shortcut::parse("Ctrl+F12").unwrap().keysym, 0xffc9);
    }

    #[test]
    fn platform_default_round_trips() {
        let binding = Shortcut::parse(DEFAULT_SHORTCUT).unwrap();
        assert_eq!(binding.value, DEFAULT_SHORTCUT);
        assert_eq!(
            binding.display_value(),
            match std::env::consts::OS {
                "macos" => "⇧⌘K",
                "windows" => "Win + Shift + K",
                _ => "⌃⇧K",
            }
        );
    }

    #[test]
    fn canonical_shortcuts_match_both_nsh_registration_and_keymap_parsers() {
        let keys = (b'A'..=b'Z')
            .chain(b'0'..=b'9')
            .map(|key| char::from(key).to_string())
            .chain(std::iter::once("Space".into()))
            .chain((1..=12).map(|n| format!("F{n}")))
            .collect::<Vec<_>>();
        for modifiers in 1..128 {
            if modifiers & !(1 | 4 | 8 | 64) != 0 || modifiers & (4 | 8 | 64) == 0 {
                continue;
            }
            for key in &keys {
                let shortcut = Shortcut::from_key(key, modifiers).unwrap();
                let keystroke = shortcut.value.to_ascii_lowercase();
                for apple in [false, true] {
                    let native = nsh_core::desktop::Shortcut::parse(&keystroke, apple).unwrap();
                    let binding =
                        nsh_core::keymap::Keystroke::parse_with(&keystroke, apple).unwrap();
                    assert_eq!(native.ctrl, modifiers & 4 != 0, "{keystroke}");
                    assert_eq!(native.alt, modifiers & 8 != 0, "{keystroke}");
                    assert_eq!(native.shift, modifiers & 1 != 0, "{keystroke}");
                    assert_eq!(native.meta, modifiers & 64 != 0, "{keystroke}");
                    assert_eq!(binding.modifiers.ctrl, native.ctrl, "{keystroke}");
                    assert_eq!(binding.modifiers.cmd, native.meta, "{keystroke}");
                }
            }
        }
    }

    #[test]
    fn shortcut_rejects_incomplete_or_unsafe_combinations() {
        for value in [
            "",
            "K",
            "Shift+K",
            "Ctrl+Ctrl+K",
            "Ctrl+",
            "Ctrl++K",
            "Ctrl+F13",
            "Ctrl+猫",
            "Ctrl+A+B",
        ] {
            assert!(Shortcut::parse(value).is_err(), "accepted {value}");
        }
    }
}
