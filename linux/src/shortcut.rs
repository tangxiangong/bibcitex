//! Persisted X11 shortcut syntax, independent of the desktop connection.
pub const DEFAULT_SHORTCUT: &str = "Ctrl+Shift+K";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shortcut {
    pub modifiers: u16,
    pub keysym: u32,
    pub value: String,
}

impl Shortcut {
    pub fn display_value(&self) -> String {
        self.value
            .replace("Super+", "⊞")
            .replace("Ctrl+", "⌃")
            .replace("Alt+", "⌥")
            .replace("Shift+", "⇧")
            .replace("Space", "␣")
    }

    pub fn from_key(key: &str, modifiers: u16) -> Result<Self, String> {
        let mut names: Vec<_> = [(64, "Super"), (4, "Ctrl"), (8, "Alt"), (1, "Shift")]
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
        let invalid = || "error.helperShortcutFormatLinux".to_owned();
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
                "SHIFT" => 1,
                "SUPER" | "WIN" | "META" => 64,
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
        let mut names: Vec<_> = [(64, "Super"), (4, "Ctrl"), (8, "Alt"), (1, "Shift")]
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
                value: DEFAULT_SHORTCUT.into(),
            }
        );
    }

    #[test]
    fn shortcut_accepts_space_digits_and_function_keys() {
        assert_eq!(Shortcut::parse("Alt+Space").unwrap().keysym, 32);
        assert_eq!(Shortcut::parse("Win+9").unwrap().value, "Super+9");
        assert_eq!(Shortcut::parse("Ctrl+F12").unwrap().keysym, 0xffc9);
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
