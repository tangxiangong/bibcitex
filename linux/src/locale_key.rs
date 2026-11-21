//! Stable, reversible keys shared by the locale compiler and native translations.
pub(crate) fn key(source: &str) -> String {
    let mut encoded = String::from("k");
    for unit in source.encode_utf16() {
        encoded.push_str(&format!("{unit:04x}"));
    }
    encoded
}
