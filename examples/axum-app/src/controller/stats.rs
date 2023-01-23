use serde_json::json;
use zino::{Application, AxumCluster, Request, Response};

pub(crate) async fn index(req: Request) -> zino::Result {
    let mut res = Response::default().provide_context(&req);
    res.set_data(json!({
        "method": "GET",
        "path": "/stats",
        "app_state_data": AxumCluster::state_data(),
        "app_sysinfo": AxumCluster::sysinfo(),
    }));
    Ok(res.into())
}
