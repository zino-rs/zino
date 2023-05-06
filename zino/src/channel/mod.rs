cfg_if::cfg_if! {
    if #[cfg(feature = "axum")] {
        pub(crate) mod axum_channel;
    }
}
