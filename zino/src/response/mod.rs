cfg_if::cfg_if! {
    if #[cfg(feature = "actix")] {
        pub(crate) mod actix_response;
    } else if #[cfg(feature = "axum")] {
        pub(crate) mod axum_response;
    } else if #[cfg(feature = "ntex")] {
        pub(crate) mod ntex_response;
    }
}
