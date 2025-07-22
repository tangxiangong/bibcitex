use crate::views::{Home, NavBar, References};
use dioxus::prelude::*;

#[derive(Debug, Clone, Routable)]
pub enum Route {
    #[layout(NavBar)]
    #[route("/")]
    Home {},
    #[route("/detail")]
    References {},
}
