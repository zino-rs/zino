use crate::router::Route;
use dioxus::prelude::*;
use dioxus_free_icons::{
    icons::{bs_icons::*, fa_brands_icons::FaRust, fa_solid_icons::FaCubes},
    Icon,
};
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
                        to: Route::StargazerList {},
                        Icon {
                            width: 16,
                            height: 16,
                            icon: BsStars,
                        }
                        span {
                            class: "ml-1",
                            "Stargazers"
                        }
                    }
                    Link {
                        class: "navbar-item",
                        to: Route::DatasetList {},
                        Icon {
                            width: 16,
                            height: 16,
                            icon: BsTable,
                        }
                        span {
                            class: "ml-1",
                            "Datasets"
                        }
                    }
                }
                div {
                    class: "navbar-end",
                    Link {
                        class: "navbar-item",
                        to: "https://github.com/photino/zino",
                        Icon {
                            width: 16,
                            height: 16,
                            icon: BsGithub,
                        }
                        span {
                            class: "ml-1",
                            "github"
                        }
                    }
                    Link {
                        class: "navbar-item",
                        to: "https://crates.io/crates/zino",
                        Icon {
                            width: 16,
                            height: 16,
                            icon: FaRust,
                        }
                        span {
                            margin_left: "0.25em",
                            "crates.io"
                        }
                    }
                    Link {
                        class: "navbar-item",
                        to: "https://docs.rs/zino",
                        Icon {
                            width: 16,
                            height: 16,
                            icon: FaCubes,
                        }
                        span {
                            class: "ml-1",
                            "docs.rs"
                        }
                    }
                }
            }
        }
        main {
            class: "my-4 px-4",
            Outlet::<Route> {}
        }
        footer {}
    }
}
