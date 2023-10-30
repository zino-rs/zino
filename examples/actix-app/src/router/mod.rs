use crate::{
    controller::{auth, file, stats, task, user},
    middleware,
    model::Tag,
};
use actix_web::web::{get, post, scope, ServiceConfig};
use zino::{DefaultController, RouterConfigure};
use zino_model::User;

pub fn routes() -> Vec<RouterConfigure> {
    vec![
        auth_router as RouterConfigure,
        file_router as RouterConfigure,
        user_router as RouterConfigure,
        tag_router as RouterConfigure,
        task_router as RouterConfigure,
    ]
}

pub fn debug_routes() -> Vec<RouterConfigure> {
    vec![stats_router as RouterConfigure]
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
            .route("/decrypt", get().to(file::decrypt)),
    );
}

fn user_router(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/user")
            .route("/new", post().to(user::new))
            .route("/{id}/delete", post().to(User::delete))
            .route("/{id}/update", post().to(User::update))
            .route("/{id}/view", get().to(user::view))
            .route("/list", get().to(User::list))
            .route("/import", post().to(User::import))
            .route("/export", get().to(User::export))
            .wrap(middleware::UserSessionInitializer),
    );
}

fn tag_router(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/tag")
            .route("/new", post().to(Tag::new))
            .route("/{id}/delete", post().to(Tag::delete))
            .route("/{id}/update", post().to(Tag::update))
            .route("/{id}/view", get().to(Tag::view))
            .route("/list", get().to(Tag::list))
            .route("/tree", get().to(Tag::tree))
            .route("/schema", get().to(Tag::schema)),
    );
}

fn task_router(cfg: &mut ServiceConfig) {
    cfg.route("/task/execute", post().to(task::execute));
}

fn stats_router(cfg: &mut ServiceConfig) {
    cfg.route("/stats", get().to(stats::index));
}
