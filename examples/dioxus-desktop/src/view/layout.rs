use crate::router::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::*;

pub fn Wrapper(cx: Scope) -> Element {
    render! {
        nav {
            class: "navbar is-link",
            div {
                class: "navbar-brand",
                Link {
                    class: "navbar-item",
                    to: Route::Home {},
                    "DataCube"
                }
            }
            div {
                class: "navbar-menu is-active",
                div {
                    class: "navbar-start",
                    Link {
                        class: "navbar-item",
                        to: Route::DatasetList {},
                        "Datasets",
                    }
                }
                div {
                    class: "navbar-end",
                    div {
                        class: "navbar-item",
                        div {
                            class: "buttons",
                            a {
                                class: "button",
                                "Sign up"
                            }
                            a {
                                class: "button is-primary",
                                strong { "Login in" }
                            }
                        }
                    }
                }
            }
        }
        Outlet::<Route> {}
        footer {}
    }
}
