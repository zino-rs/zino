use crate::{
    controller::{auth, stats, task, user},
    middleware,
};
use axum::{
    middleware::from_fn,
    routing::{get, post},
    Router,
};
use zino::DefaultController;
use zino_model::{Tag, User};

pub fn routes() -> Vec<Router> {
    let mut routes = Vec::new();

    // Auth controller.
    let router = Router::new().route("/auth/login", post(auth::login));
    routes.push(router);

    // User controller.
    let router = Router::new()
        .route("/user/new", post(user::new))
        .route("/user/:id/delete", post(User::delete))
        .route("/user/:id/update", post(User::update))
        .route("/user/:id/view", get(user::view))
        .route("/user/list", get(User::list))
        .route("/user/import", post(User::import))
        .route("/user/export", get(User::export));
    routes.push(router);

    // Tag controller.
    let router = Router::new()
        .route("/tag/new", post(Tag::new))
        .route("/tag/:id/delete", post(Tag::delete))
        .route("/tag/:id/update", post(Tag::update))
        .route("/tag/:id/view", get(Tag::view))
        .route("/tag/list", get(Tag::list));
    routes.push(router);

    // Task controller.
    let router = Router::new().route("/task/execute", post(task::execute));
    routes.push(router);

    // Stats controller.
    let router = Router::new()
        .route("/stats", get(stats::index))
        .layer(from_fn(middleware::check_client_ip));
    routes.push(router);

    routes
}
