use crate::components::Database;
use dioxus::prelude::*;
use std::{collections::BTreeMap, path::PathBuf, str::FromStr};

#[component]
pub fn Home() -> Element {
    let db = use_signal(|| {
        let mut map = BTreeMap::new();
        let path = PathBuf::from_str("./database.bib").unwrap();
        map.insert("test".to_owned(), path);
        map
    });

    rsx! {
        Database { db: db() }
    }
}
