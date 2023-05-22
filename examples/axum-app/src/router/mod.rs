use crate::{
    controller::{bench, stats, task, user},
    middleware,
};
use axum::{
    middleware::from_fn,
    routing::{get, post},
    Router,
};

pub fn routes() -> Vec<Router> {
    let mut routes = Vec::new();

    // User controller.
    let router = Router::new()
        .route("/user/new", post(user::new))
        .route("/user/:id/update", post(user::update))
        .route("/user/list", get(user::list))
        .route("/user/:id/view", get(user::view));
    routes.push(router);

    // Task controller.
    let router = Router::new().route("/task/execute", post(task::execute));
    routes.push(router);

    // Stats controller.
    let router = Router::new()
        .route("/stats", get(stats::index))
        .layer(from_fn(middleware::check_client_ip));
    routes.push(router);

    // Bench controller.
    let router = Router::new().route("/bench/rbatis/user/:id/view", get(bench::rbatis_user_view));
    routes.push(router);

    routes
}
