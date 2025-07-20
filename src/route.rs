use crate::views::{Home, NavBar};
use dioxus::prelude::*;

#[derive(Debug, Clone, Routable)]
pub enum Route {
    #[layout(NavBar)]
    #[route("/")]
    Home {},
}
