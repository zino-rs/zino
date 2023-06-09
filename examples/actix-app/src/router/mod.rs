use crate::{
    controller::{bench, stats, task, user},
    model::User,
};
use actix_web::web::{get, post, ServiceConfig};
use zino::{DefaultController, RouterConfigure};
use zino_model::Tag;

pub fn routes() -> Vec<RouterConfigure> {
    vec![
        user_router as RouterConfigure,
        tag_router as RouterConfigure,
        task_router as RouterConfigure,
        stats_router as RouterConfigure,
        bench_router as RouterConfigure,
    ]
}

fn user_router(cfg: &mut ServiceConfig) {
    cfg.route("/user/new", post().to(user::new))
        .route("/user/{id}/delete", post().to(User::delete))
        .route("/user/{id}/update", post().to(User::update))
        .route("/user/{id}/view", get().to(user::view))
        .route("/user/list", get().to(User::list));
}

fn tag_router(cfg: &mut ServiceConfig) {
    cfg.route("/tag/new", post().to(Tag::new))
        .route("/tag/{id}/delete", post().to(Tag::delete))
        .route("/tag/{id}/update", post().to(Tag::update))
        .route("/tag/{id}/view", get().to(Tag::view))
        .route("/tag/list", get().to(Tag::list));
}

fn task_router(cfg: &mut ServiceConfig) {
    cfg.route("/task/execute", post().to(task::execute));
}

fn stats_router(cfg: &mut ServiceConfig) {
    cfg.route("/stats", get().to(stats::index));
}

fn bench_router(cfg: &mut ServiceConfig) {
    cfg.route(
        "/bench/rbatis/user/{id}/view",
        get().to(bench::rbatis_user_view),
    );
}
