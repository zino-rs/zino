use std::{sync::LazyLock, time::Duration};
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer, ExposeHeaders};
use zino_core::application::Application;

// CORS middleware.
pub(crate) static CORS_MIDDLEWARE: LazyLock<CorsLayer> = LazyLock::new(|| {
    let config = crate::AxumCluster::config();
    match config.get("cors").and_then(|t| t.as_table()) {
        Some(cors) => {
            let allow_credentials = cors
                .get("allow-credentials")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let allow_origin = cors
                .get("allow-origin")
                .and_then(|v| v.as_array())
                .map(|values| {
                    let origins = values
                        .iter()
                        .filter_map(|v| v.as_str().and_then(|s| s.parse().ok()))
                        .collect::<Vec<_>>();
                    AllowOrigin::list(origins)
                })
                .unwrap_or_else(AllowOrigin::mirror_request);
            let allow_methods = cors
                .get("allow-methods")
                .and_then(|v| v.as_array())
                .map(|values| {
                    let methods = values
                        .iter()
                        .filter_map(|v| v.as_str().and_then(|s| s.parse().ok()))
                        .collect::<Vec<_>>();
                    AllowMethods::list(methods)
                })
                .unwrap_or_else(AllowMethods::mirror_request);
            let allow_headers = cors
                .get("allow-headers")
                .and_then(|v| v.as_array())
                .map(|values| {
                    let header_names = values
                        .iter()
                        .filter_map(|v| v.as_str().and_then(|s| s.parse().ok()))
                        .collect::<Vec<_>>();
                    AllowHeaders::list(header_names)
                })
                .unwrap_or_else(AllowHeaders::mirror_request);
            let expose_headers = cors
                .get("expose-headers")
                .and_then(|v| v.as_array())
                .map(|values| {
                    let header_names = values
                        .iter()
                        .filter_map(|v| v.as_str().and_then(|s| s.parse().ok()))
                        .collect::<Vec<_>>();
                    ExposeHeaders::list(header_names)
                })
                .unwrap_or_else(ExposeHeaders::any);
            let max_age = cors
                .get("max-age")
                .and_then(|v| v.as_integer().and_then(|i| u64::try_from(i).ok()))
                .unwrap_or(60 * 60);
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
