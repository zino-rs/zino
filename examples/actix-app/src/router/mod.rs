use crate::controller::{stats, task, user};
use actix_web::web::{get, post, ServiceConfig};
use zino::RouterConfigure;

pub(crate) fn routes() -> Vec<RouterConfigure> {
    vec![
        configure_user as RouterConfigure,
        configure_task as RouterConfigure,
        configure_stats as RouterConfigure,
    ]
}

fn configure_user(cfg: &mut ServiceConfig) {
    cfg.route("/user/new", get().to(user::new))
        .route("/user/{id}/update", post().to(user::update))
        .route("/user/list", get().to(user::list))
        .route("/user/{id}/view", get().to(user::view));
}

fn configure_task(cfg: &mut ServiceConfig) {
    cfg.route("/task/execute", post().to(task::execute));
}

fn configure_stats(cfg: &mut ServiceConfig) {
    cfg.route("/stats", get().to(stats::index));
}
