use crate::router::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::*;

pub fn Wrapper(cx: Scope) -> Element {
    render! {
        header {
            nav {
                ul {
                    li { Link { to: Route::Home {}, "Home" } }
                    li { Link { to: Route::DatasetList {}, "Datasets" } }
                }
            }
        }
        Outlet::<Route> {}
        footer {}
    }
}
