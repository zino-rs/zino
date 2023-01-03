use crate::controller::{stats, user};
use axum::{
    routing::{get, post},
    Router,
};
use std::collections::HashMap;

pub(crate) fn init() -> HashMap<&'static str, Router> {
    let mut parties = HashMap::new();

    // User controller.
    let controller = Router::new()
        .route("/new", post(user::new))
        .route("/update", post(user::update))
        .route("/list", get(user::list))
        .route("/:id/view", get(user::view));
    parties.insert("/user", controller);

    // Stats controller.
    let controller = Router::new().route("/", get(stats::index));
    parties.insert("/stats", controller);

    parties
}
