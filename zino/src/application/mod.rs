cfg_if::cfg_if! {
    if #[cfg(feature = "actix")] {
        mod plugin_loader;
        pub(crate) mod actix_cluster;

        use plugin_loader::load_plugins;
    } else if #[cfg(feature = "axum")] {
        mod plugin_loader;
        pub(crate) mod axum_cluster;

        use plugin_loader::load_plugins;
    } else if #[cfg(feature = "dioxus-desktop")] {
        mod plugin_loader;
        pub(crate) mod dioxus_desktop;

        use plugin_loader::load_plugins;
    } else if #[cfg(feature = "dioxus-ssr")] {
        mod plugin_loader;
        pub(crate) mod dioxus_ssr;

        use plugin_loader::load_plugins;
    } else if #[cfg(feature = "ntex")] {
        mod plugin_loader;
        pub(crate) mod ntex_cluster;

        use plugin_loader::load_plugins;
    }
}
