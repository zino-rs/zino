use actix_cors::Cors;
use zino_core::{application::Application, extension::TomlTableExt};

/// CORS middleware.
pub(crate) fn cors_middleware() -> Cors {
    if let Some(config) = crate::Cluster::config().get_table("cors") {
        let max_age = config.get_usize("max-age").unwrap_or(60 * 60);
        let mut cors = Cors::default().max_age(max_age);
        if let Some(origin) = config.get_str("allow-origin") {
            cors = cors.allowed_origin(origin);
        }
        if let Some(methods) = config.get_str_array("allow-methods") {
            cors = cors.allowed_methods(methods);
        }
        if let Some(headers) = config.get_str_array("allow-headers") {
            cors = cors.allowed_headers(headers);
        }
        if let Some(headers) = config.get_str_array("expose-headers") {
            cors = cors.expose_headers(headers);
        }
        if config.get_bool("allow-credentials") == Some(true) {
            cors = cors.supports_credentials();
        }
        cors
    } else {
        Cors::permissive()
    }
}
