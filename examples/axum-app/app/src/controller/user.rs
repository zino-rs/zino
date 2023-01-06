use serde_json::json;
use zino::Request;
use zino_core::{Model, Query, Rejection, RequestContext, Response, Schema, Uuid};
use zino_model::User;

pub(crate) async fn new(mut req: Request) -> zino::Result {
    let mut user = User::new();
    let mut res = req.model_validation(&mut user).await?;

    let rows = user.upsert().await.unwrap();
    let data = json!({
        "method": req.request_method(),
        "path": req.request_path(),
        "rows": rows,
    });
    res.set_data(data);
    Ok(res.into())
}

pub(crate) async fn update(mut req: Request) -> zino::Result {
    let mut user = User::new();
    let validation = req.parse_body().await.map(|body| user.read_map(body))?;
    let res = Response::from(validation);
    Ok(res.into())
}

pub(crate) async fn list(req: Request) -> zino::Result {
    let mut query = Query::new();
    let mut res = req.query_validation(&mut query)?;

    let users = User::find(query)
        .await
        .map_err(Rejection::internal_server_error)?;
    let data = json!({
        "users": users,
    });
    res.set_data(data);
    Ok(res.into())
}

pub(crate) async fn view(mut req: Request) -> zino::Result {
    let user_id = req.parse_params::<Uuid>().await?;
    let mut query = Query::new();
    let mut res = req.query_validation(&mut query)?;
    query.insert_filter("id", user_id.to_string());

    let message = json!({
        "path": req.request_path(),
    });
    let event = req.cloud_event("message", message);
    req.try_send(event)?;

    let user = User::find_one(query)
        .await
        .map_err(Rejection::internal_server_error)?;
    let data = json!({
        "user": user,
    });
    res.set_data(data);
    Ok(res.into())
}
