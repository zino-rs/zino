use crate::service::user;
use serde_json::json;
use zino::{ExtractRejection, Map, Model, Request, RequestContext, Response, Result, Schema, Uuid};
use zino_model::User;

pub(crate) async fn new(mut req: Request) -> Result {
    let mut user = User::new();
    let mut res: Response = req.model_validation(&mut user).await?;

    user.upsert().await.extract_with_context(&req)?;
    let data = json!({
        "method": req.request_method().as_ref(),
        "path": req.request_path(),
    });
    res.set_data(&data);
    Ok(res.into())
}

pub(crate) async fn update(mut req: Request) -> Result {
    let user_id: Uuid = req.parse_param("id")?;
    let body: Map = req.parse_body().await?;
    let (validation, data) = user::update(user_id, body)
        .await
        .extract_with_context(&req)?;
    let mut res = Response::from(validation).provide_context(&req);
    res.set_data(&data);
    Ok(res.into())
}

pub(crate) async fn list(req: Request) -> Result {
    let mut query = User::default_query();
    let mut res: Response = req.query_validation(&mut query)?;
    let users: Vec<Map> = User::find(&query).await.extract_with_context(&req)?;
    let data = json!({
        "users": users,
    });
    res.set_data(&data);
    Ok(res.into())
}

pub(crate) async fn view(req: Request) -> Result {
    let locale_cookie = req.new_cookie("locale", "en-US", None);
    req.add_cookie(locale_cookie);

    let user_id: Uuid = req.parse_param("id")?;
    let mut query = User::default_query();
    let mut res: Response = req.query_validation(&mut query)?;
    query.add_filter("id", user_id.to_string());

    let message = json!({
        "path": req.request_path(),
    });
    let event = req.cloud_event("message", message);
    req.try_send(event)?;

    let (db_query_duration, data) = user::view(&req, &query).await.extract_with_context(&req)?;
    res.record_server_timing("db", None, Some(db_query_duration));
    res.set_data(&data);
    Ok(res.into())
}
