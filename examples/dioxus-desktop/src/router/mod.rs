use crate::controller::home::Homepage;
use dioxus::prelude::*;
use dioxus_router::prelude::*;

#[derive(Clone, PartialEq, Eq, Routable)]
pub enum Route {
    #[route("/")]
    Homepage {},
}

impl Default for Route {
    fn default() -> Self {
        Self::Homepage {}
    }
}
