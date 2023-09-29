cfg_if::cfg_if! {
    if #[cfg(feature = "actix")] {
        mod actix_context;
        mod actix_cors;
        mod actix_etag;
        mod actix_tracing;

        pub(crate) use self::actix_context::RequestContextInitializer;
        pub(crate) use self::actix_cors::cors_middleware;
        pub(crate) use self::actix_etag::ETagFinalizer;
        pub(crate) use self::actix_tracing::tracing_middleware;
    } else if #[cfg(feature = "axum")] {
        mod axum_context;
        mod axum_etag;
        mod axum_static_pages;
        mod tower_cors;
        mod tower_tracing;

        pub(crate) use self::axum_context::request_context;
        pub(crate) use self::axum_etag::extract_etag;
        pub(crate) use self::axum_static_pages::serve_static_pages;
        pub(crate) use self::tower_cors::CORS_MIDDLEWARE;
        pub(crate) use self::tower_tracing::TRACING_MIDDLEWARE;
    }
}
