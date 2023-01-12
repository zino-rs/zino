cfg_if::cfg_if! {
    if #[cfg(feature = "axum-server")] {
        pub(crate) mod axum_sse;
        pub(crate) mod axum_websocket;
    }
}
