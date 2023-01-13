use serde_json::json;
use zino::{Request, RequestContext, Response};

pub(crate) async fn index(req: Request) -> zino::Result {
    let mut res = Response::default();
    res.set_data(json!({
        "method": "GET",
        "path": "/stats",
        "config": req.config(),
    }));
    Ok(res.into())
}
