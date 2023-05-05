use crate::controller::{stats, task, user};
use actix_web::web::{get, post, ServiceConfig};
use zino::RouterConfigure;

pub(crate) fn routes() -> Vec<RouterConfigure> {
    vec![
        user_router as RouterConfigure,
        task_router as RouterConfigure,
        stats_router as RouterConfigure,
    ]
}

fn user_router(cfg: &mut ServiceConfig) {
    cfg.route("/user/new", get().to(user::new))
        .route("/user/{id}/update", post().to(user::update))
        .route("/user/list", get().to(user::list))
        .route("/user/{id}/view", get().to(user::view));
}

fn task_router(cfg: &mut ServiceConfig) {
    cfg.route("/task/execute", post().to(task::execute));
}

fn stats_router(cfg: &mut ServiceConfig) {
    cfg.route("/stats", get().to(stats::index));
}
