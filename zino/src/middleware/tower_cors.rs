use std::{sync::LazyLock, time::Duration};
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer, ExposeHeaders};
use zino_core::State;

// CORS middleware.
pub(crate) static CORS_MIDDLEWARE: LazyLock<CorsLayer> = LazyLock::new(|| {
    let shared_state = State::shared();
    match shared_state.config().get("cors").and_then(|t| t.as_table()) {
        Some(cors) => {
            let allow_credentials = cors
                .get("allow-credentials")
                .and_then(|t| t.as_bool())
                .unwrap_or(false);
            let allow_origin = cors
                .get("allow-origin")
                .and_then(|t| t.as_array())
                .map(|v| {
                    let origins = v
                        .iter()
                        .filter_map(|t| t.as_str().and_then(|s| s.parse().ok()))
                        .collect::<Vec<_>>();
                    AllowOrigin::list(origins)
                })
                .unwrap_or(AllowOrigin::mirror_request());
            let allow_methods = cors
                .get("allow-methods")
                .and_then(|t| t.as_array())
                .map(|v| {
                    let methods = v
                        .iter()
                        .filter_map(|t| t.as_str().and_then(|s| s.parse().ok()))
                        .collect::<Vec<_>>();
                    AllowMethods::list(methods)
                })
                .unwrap_or(AllowMethods::mirror_request());
            let allow_headers = cors
                .get("allow-headers")
                .and_then(|t| t.as_array())
                .map(|v| {
                    let header_names = v
                        .iter()
                        .filter_map(|t| t.as_str().and_then(|s| s.parse().ok()))
                        .collect::<Vec<_>>();
                    AllowHeaders::list(header_names)
                })
                .unwrap_or(AllowHeaders::mirror_request());
            let expose_headers = cors
                .get("expose-headers")
                .and_then(|t| t.as_array())
                .map(|v| {
                    let header_names = v
                        .iter()
                        .filter_map(|t| t.as_str().and_then(|s| s.parse().ok()))
                        .collect::<Vec<_>>();
                    ExposeHeaders::list(header_names)
                })
                .unwrap_or(ExposeHeaders::any());
            let max_age = cors
                .get("max-age")
                .and_then(|t| t.as_integer().and_then(|i| i.try_into().ok()))
                .unwrap_or(86400);
            CorsLayer::new()
                .allow_credentials(allow_credentials)
                .allow_origin(allow_origin)
                .allow_methods(allow_methods)
                .allow_headers(allow_headers)
                .expose_headers(expose_headers)
                .max_age(Duration::from_secs(max_age))
        }
        None => CorsLayer::permissive(),
    }
});
