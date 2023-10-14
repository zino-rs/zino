use crate::router::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::*;
use zino::prelude::*;

pub fn Dataset(cx: Scope) -> Element {
    render! {
        h1 { "Datasets" }
        Outlet::<Route> {}
    }
}

pub fn DatasetList(cx: Scope) -> Element {
    render! {
        ul {
            li {
                Link {
                    to: Route::DatasetView { id: Uuid::new_v4() },
                    "View the first dataset"
                }
            }
            li {
                Link {
                    to: Route::DatasetView { id: Uuid::new_v4() },
                    "View the second dataset"
                }
            }
        }
    }
}

#[inline_props]
pub fn DatasetView(cx: Scope, id: Uuid) -> Element {
    render! {
        h2 { "Dataset ID: {id}"}
    }
}
