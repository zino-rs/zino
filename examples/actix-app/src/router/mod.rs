use crate::{
    controller::{auth, stats, task, user},
    middleware,
};
use actix_web::web::{get, post, scope, ServiceConfig};
use zino::{DefaultController, RouterConfigure};
use zino_model::{Tag, User};

pub fn routes() -> Vec<RouterConfigure> {
    vec![
        auth_router as RouterConfigure,
        user_router as RouterConfigure,
        tag_router as RouterConfigure,
        task_router as RouterConfigure,
        stats_router as RouterConfigure,
    ]
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
            .wrap(middleware::UserSessionInitializer),
    );
}

fn task_router(cfg: &mut ServiceConfig) {
    cfg.route("/task/execute", post().to(task::execute));
}

fn stats_router(cfg: &mut ServiceConfig) {
    cfg.route("/stats", get().to(stats::index));
}
