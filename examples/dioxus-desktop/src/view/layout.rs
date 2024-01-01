use crate::router::Route;
use dioxus::prelude::*;
use dioxus_free_icons::{
    icons::{bs_icons::*, fa_brands_icons::FaRust, fa_solid_icons::FaCubes},
    Icon as SvgIcon,
};
use dioxus_router::prelude::*;
use zino_dioxus::prelude::*;

pub fn Wrapper(cx: Scope) -> Element {
    render! {
        Navbar {
            NavbarStart {
                NavbarLink {
                    to: Route::Overview {},
                    IconText {
                        Icon {
                            SvgIcon { icon: BsSpeedometer2 }
                        }
                        span { "Overview" }
                    }
                }
                NavbarLink {
                    to: Route::StargazerList {},
                    IconText {
                        Icon {
                            SvgIcon { icon: BsStars }
                        }
                        span { "Stargazers" }
                    }
                }
                NavbarLink {
                    to: Route::DependencyList {},
                    IconText {
                        Icon {
                            SvgIcon { icon: BsBricks }
                        }
                        span { "Dependencies" }
                    }
                }
            }
            NavbarEnd {
                NavbarLink {
                    to: "https://github.com/zino-rs/zino",
                    IconText {
                        Icon {
                            SvgIcon { icon: BsGithub }
                        }
                        span { "github" }
                    }
                }
                NavbarLink {
                    to: "https://crates.io/crates/zino",
                    IconText {
                        Icon {
                            SvgIcon { icon: FaRust }
                        }
                        span { "crates.io" }
                    }
                }
                NavbarLink {
                    to: "https://docs.rs/zino",
                    IconText {
                        Icon {
                            SvgIcon { icon: FaCubes }
                        }
                        span { "docs.rs" }
                    }
                }
            }
        }
        MainContainer {
            Outlet::<Route> {}
        }
    }
}
