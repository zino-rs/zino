use serde_json::json;
use zino::{Application, AxumCluster, Request, Response};

pub(crate) async fn index(req: Request) -> zino::Result {
    let mut res = Response::default().provide_context(&req);
    let stats = json!({
        "method": "GET",
        "path": "/stats",
        "app_state_data": AxumCluster::state_data(),
        "app_sysinfo": AxumCluster::sysinfo(),
    });
    res.set_data(json!({
        "title": "Stats",
        "stats": serde_json::to_string_pretty(&stats).unwrap_or_default(),
    }));
    Ok(res.render("stats.html").into())
}
