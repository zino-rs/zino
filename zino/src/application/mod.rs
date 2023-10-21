cfg_if::cfg_if! {
    if #[cfg(feature = "actix")] {
        pub(crate) mod actix_cluster;
    } else if #[cfg(feature = "axum")] {
        pub(crate) mod axum_cluster;
    } else if #[cfg(feature = "dioxus-desktop")] {
        pub(crate) mod dioxus_desktop;
    }
}
