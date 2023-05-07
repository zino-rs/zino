use actix_cors::Cors;
use actix_web::http::{header::HeaderName, Method};
use zino_core::{application::Application, extension::TomlTableExt};

/// CORS middleware.
pub(crate) fn cors_middleware() -> Cors {
    if let Some(cors) = crate::Cluster::config().get_table("cors") {
        let allow_methods = cors
            .get_array("allow-methods")
            .map(|values| {
                values
                    .iter()
                    .filter_map(|v| v.as_str().and_then(|s| s.parse::<Method>().ok()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let allow_headers = cors
            .get_array("allow-headers")
            .map(|values| {
                values
                    .iter()
                    .filter_map(|v| v.as_str().and_then(|s| s.parse::<HeaderName>().ok()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let expose_headers = cors
            .get_array("expose-headers")
            .map(|values| {
                values
                    .iter()
                    .filter_map(|v| v.as_str().and_then(|s| s.parse::<HeaderName>().ok()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let max_age = cors.get_usize("max-age").unwrap_or(60 * 60);
        Cors::default()
            .allow_any_origin()
            .allowed_methods(allow_methods)
            .allowed_headers(allow_headers)
            .expose_headers(expose_headers)
            .max_age(max_age)
    } else {
        Cors::permissive()
    }
}
