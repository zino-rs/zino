use std::{sync::LazyLock, time::Duration};
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer, ExposeHeaders};
use zino_core::{application::Application, extension::TomlTableExt};

// CORS middleware.
pub(crate) static CORS_MIDDLEWARE: LazyLock<CorsLayer> = LazyLock::new(|| {
    if let Some(cors) = crate::AxumCluster::config().get_table("cors") {
        let allow_credentials = cors.get_bool("allow-credentials").unwrap_or(false);
        let allow_origin = cors
            .get_array("allow-origin")
            .map(|values| {
                let origins = values
                    .iter()
                    .filter_map(|v| v.as_str().and_then(|s| s.parse().ok()))
                    .collect::<Vec<_>>();
                AllowOrigin::list(origins)
            })
            .unwrap_or_else(AllowOrigin::mirror_request);
        let allow_methods = cors
            .get_array("allow-methods")
            .map(|values| {
                let methods = values
                    .iter()
                    .filter_map(|v| v.as_str().and_then(|s| s.parse().ok()))
                    .collect::<Vec<_>>();
                AllowMethods::list(methods)
            })
            .unwrap_or_else(AllowMethods::mirror_request);
        let allow_headers = cors
            .get_array("allow-headers")
            .map(|values| {
                let header_names = values
                    .iter()
                    .filter_map(|v| v.as_str().and_then(|s| s.parse().ok()))
                    .collect::<Vec<_>>();
                AllowHeaders::list(header_names)
            })
            .unwrap_or_else(AllowHeaders::mirror_request);
        let expose_headers = cors
            .get_array("expose-headers")
            .map(|values| {
                let header_names = values
                    .iter()
                    .filter_map(|v| v.as_str().and_then(|s| s.parse().ok()))
                    .collect::<Vec<_>>();
                ExposeHeaders::list(header_names)
            })
            .unwrap_or_else(ExposeHeaders::any);
        let max_age = cors
            .get_duration("max-age")
            .unwrap_or_else(|| Duration::from_secs(60 * 60));
        CorsLayer::new()
            .allow_credentials(allow_credentials)
            .allow_origin(allow_origin)
            .allow_methods(allow_methods)
            .allow_headers(allow_headers)
            .expose_headers(expose_headers)
            .max_age(max_age)
    } else {
        CorsLayer::permissive()
    }
});
