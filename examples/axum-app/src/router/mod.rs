use crate::{
    controller::{auth, file, stats, user},
    middleware,
    model::{Tag, User},
};
use axum::{
    Router,
    middleware::from_fn,
    routing::{get, post},
};
use zino::DefaultController;

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
        .route("/file/decrypt", get(file::decrypt))
        .layer(from_fn(middleware::init_user_session));
    routes.push(router);

    // User controller.
    let router = Router::new()
        .route("/user/new", post(user::new))
        .route("/user/{id}/delete", post(User::soft_delete))
        .route("/user/{id}/update", post(User::update))
        .route("/user/{id}/view", get(user::view))
        .route("/user/list", get(User::list))
        .route("/user/import", post(User::import))
        .route("/user/export", get(User::export))
        .route("/user/stats", get(user::stats));
    routes.push(router);

    // Tag controller.
    let router = Router::new()
        .route("/tag/new", post(Tag::new))
        .route("/tag/{id}/delete", post(Tag::soft_delete))
        .route("/tag/{id}/update", post(Tag::update))
        .route("/tag/{id}/view", get(Tag::view))
        .route("/tag/list", get(Tag::list))
        .route("/tag/tree", get(Tag::tree))
        .layer(from_fn(middleware::check_admin_role))
        .layer(from_fn(middleware::init_user_session));
    routes.push(router);

    routes
}

pub fn debug_routes() -> Vec<Router> {
    let mut routes = Vec::new();

    // Stats controller.
    let router = Router::new()
        .route("/stats", get(stats::index))
        .route("/stats/app_state", get(stats::app_state));
    routes.push(router);

    // User schema controller.
    let router = Router::new()
        .route("/user/schema", get(User::schema))
        .route("/user/definition", get(User::definition))
        .route("/user/mock", get(User::mock));
    routes.push(router);

    // Tag schema controller.
    let router = Router::new()
        .route("/tag/schema", get(Tag::schema))
        .route("/tag/definition", get(Tag::definition))
        .route("/tag/mock", get(Tag::mock));
    routes.push(router);

    routes
}
