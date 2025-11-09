use crate::{
    controller::{auth, file, stats, user},
    middleware,
    model::{Tag, User},
};
use actix_web::web::{ServiceConfig, get, post, scope};
use zino::{DefaultController, RouterConfigure};

pub fn routes() -> Vec<RouterConfigure> {
    vec![auth_router, file_router, user_router, tag_router]
}

pub fn debug_routes() -> Vec<RouterConfigure> {
    vec![stats_router, user_debug_router, tag_debug_router]
}

fn auth_router(cfg: &mut ServiceConfig) {
    cfg.route("/auth/login", post().to(auth::login));
    cfg.service(
        scope("/auth")
            .route("/refresh", get().to(auth::refresh))
            .route("/logout", post().to(auth::logout))
            .wrap(middleware::UserSessionInitializer),
    );
}

fn file_router(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/file")
            .route("/upload", post().to(file::upload))
            .route("/decrypt", get().to(file::decrypt))
            .wrap(middleware::UserSessionInitializer),
    );
}

fn user_router(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/user")
            .route("/new", post().to(user::new))
            .route("/{id}/delete", post().to(User::soft_delete))
            .route("/{id}/update", post().to(User::update))
            .route("/{id}/view", get().to(user::view))
            .route("/list", get().to(User::list))
            .route("/import", post().to(User::import))
            .route("/export", get().to(User::export))
            .wrap(middleware::UserSessionInitializer),
    );
}

fn tag_router(cfg: &mut ServiceConfig) {
    cfg.route("/tag/new", post().to(Tag::new))
        .route("/tag/{id}/delete", post().to(Tag::soft_delete))
        .route("/tag/{id}/update", post().to(Tag::update))
        .route("/tag/{id}/view", get().to(Tag::view))
        .route("/tag/list", get().to(Tag::list))
        .route("/tag/tree", get().to(Tag::tree));
}

fn stats_router(cfg: &mut ServiceConfig) {
    cfg.route("/stats", get().to(stats::index));
}

fn user_debug_router(cfg: &mut ServiceConfig) {
    cfg.route("/user/schema", get().to(User::schema))
        .route("/user/definition", get().to(User::definition))
        .route("/user/mock", get().to(User::mock));
}

fn tag_debug_router(cfg: &mut ServiceConfig) {
    cfg.route("/tag/schema", get().to(Tag::schema))
        .route("/tag/definition", get().to(Tag::definition))
        .route("/tag/mock", get().to(Tag::mock));
}
