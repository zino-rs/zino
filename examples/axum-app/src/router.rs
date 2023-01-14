use crate::controller::{stats, user};
use axum::{
    routing::{get, post},
    Router,
};
use std::collections::HashMap;

pub(crate) fn init_routes() -> HashMap<&'static str, Router> {
    let mut routes = HashMap::new();

    // User controller.
    let controller = Router::new()
        .route("/new", post(user::new))
        .route("/update", post(user::update))
        .route("/list", get(user::list))
        .route("/:id/view", get(user::view));
    routes.insert("/user", controller);

    // Stats controller.
    let controller = Router::new().route("/", get(stats::index));
    routes.insert("/stats", controller);

    routes
}
