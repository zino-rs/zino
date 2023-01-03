use crate::{Request, RequestContext, Response};
use serde_json::json;

pub(crate) async fn index(req: Request) -> crate::Result {
    let mut res = Response::default();
    res.set_data(json!({
        "method": "GET",
        "path": "/stats",
        "config": req.config(),
    }));
    Ok(res.into())
}
