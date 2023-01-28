use fluent::fluent_args;
use serde_json::json;
use std::time::Instant;
use zino::{Model, Query, Rejection, Request, RequestContext, Response, Schema, Uuid};
use zino_model::User;

pub(crate) async fn new(mut req: Request) -> zino::Result {
    let mut user = User::new();
    let mut res = req.model_validation(&mut user).await?;

    let rows = user
        .upsert()
        .await
        .map_err(Rejection::internal_server_error)?;
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
    let res = Response::from(validation).provide_context(&req);
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
    let user_id = req.parse_param::<Uuid>("id")?;
    let mut query = Query::new();
    let mut res = req.query_validation(&mut query)?;
    query.insert_filter("id", user_id.to_string());

    let message = json!({
        "path": req.request_path(),
    });
    let event = req.cloud_event("message", message);
    req.try_send(event)?;

    let db_query_start_time = Instant::now();
    let user = User::find_one(query)
        .await
        .map_err(Rejection::internal_server_error)?
        .ok_or_else(|| Rejection::not_found("user does not exits"))?;
    res.record_server_timing("db", None, db_query_start_time.elapsed());

    let args = fluent_args![
        "name" => user.get("name").and_then(|v| v.as_str()).unwrap_or_default()
    ];
    let user_intro = req
        .translate("user-intro", args)
        .map_err(Rejection::internal_server_error)?;
    let data = json!({
        "user": user,
        "intro": user_intro,
    });
    res.set_data(data);
    Ok(res.into())
}
