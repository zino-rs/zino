use crate::{
    controller::{auth, file, stats, task, user},
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
    let router = Router::new().route("/auth/login", post(auth::login)).merge(
        Router::new()
            .route("/auth/refresh", get(auth::refresh))
            .route("/auth/logout", post(auth::logout))
            .layer(from_fn(middleware::init_user_session)),
    );
    routes.push(router);

    // File controller.
    let router = Router::new()
        .route("/file/upload", post(file::upload))
        .route("/file/decrypt", get(file::decrypt));
    routes.push(router);

    // User controller.
    let router = Router::new()
        .route("/user/new", post(user::new))
        .route("/user/:id/delete", post(User::delete))
        .route("/user/:id/update", post(User::update))
        .route("/user/:id/view", get(user::view))
        .route("/user/list", get(User::list))
        .route("/user/import", post(User::import))
        .route("/user/export", get(User::export))
        .layer(from_fn(middleware::init_user_session));
    routes.push(router);

    // Tag controller.
    let router = Router::new()
        .route("/tag/new", post(Tag::new))
        .route("/tag/:id/delete", post(Tag::delete))
        .route("/tag/:id/update", post(Tag::update))
        .route("/tag/:id/view", get(Tag::view))
        .route("/tag/list", get(Tag::list))
        .route("/tag/schema", get(Tag::schema));
    routes.push(router);

    // Task controller.
    let router = Router::new().route("/task/execute", post(task::execute));
    routes.push(router);

    routes
}

pub fn debug_routes() -> Vec<Router> {
    let mut routes = Vec::new();

    // Stats controller.
    let router = Router::new().route("/stats", get(stats::index));
    routes.push(router);

    routes
}
