use crate::Result;
use once_cell::sync::Lazy;
use std::collections::BTreeMap;

/// A static map of MSC codes to their corresponding text descriptions.
pub static MSC_MAP: Lazy<BTreeMap<String, String>> = Lazy::new(|| load_data().unwrap_or_default());

fn load_data() -> Result<BTreeMap<String, String>> {
    let content = include_str!("../../data/msc.csv");
    let mut reader = csv::Reader::from_reader(content.as_bytes());
    let mut map = BTreeMap::new();
    for result in reader.records() {
        let record = result?;
        let code = record[0].trim().to_string();
        let text = record[1].trim().to_string();
        map.insert(code, text);
    }
    Ok(map)
}

/// Parses a str into vectors of MSC code strings.
///
/// # Examples
/// "60K50 (26A33 35R11 60H30)" -> ["60K50", "26A33", "35R11", "60H30"]
///
/// "26A33" -> ["26A33"]
pub fn parse_code(raw: &str) -> Vec<String> {
    raw.chars()
        .collect::<String>()
        .replace(['(', ')'], " ")
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_data() {
        let map = load_data().unwrap();
        assert_eq!(map.len(), 6603);
        assert_eq!(
            map.get("97M70").unwrap(),
            "Behavioral and social sciences (aspects of mathematics education)"
        )
    }

    #[test]
    fn test_parse_code() {
        assert_eq!(
            parse_code("60K50 (26A33 35R11 60H30)"),
            vec!["60K50", "26A33", "35R11", "60H30"]
        );
        assert_eq!(parse_code("26A33"), vec!["26A33"]);
    }
}
