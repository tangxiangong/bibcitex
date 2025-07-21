use crate::views::{Detail, Home, NavBar};
use dioxus::prelude::*;

#[derive(Debug, Clone, Routable)]
pub enum Route {
    #[layout(NavBar)]
    #[route("/")]
    Home {},
    #[route("/detail")]
    Detail {},
}
