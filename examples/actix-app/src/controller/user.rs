use std::time::Instant;
use zino::{prelude::*, Request, Response, Result};
use zino_model::user::User;

pub async fn new(mut req: Request) -> Result {
    let mut user = User::new();
    let mut res = req.model_validation(&mut user).await?;
    let validation = user.check_constraints().await.extract(&req)?;
    if !validation.is_success() {
        reject!(req, validation);
    }

    let user_name = user.name().to_owned();
    user.insert().await.extract(&req)?;

    let args = fluent_args![
        "name" => user_name
    ];
    let user_intro = req.translate("user-intro", Some(args)).extract(&req)?;
    let data = json!({
        "method": req.request_method(),
        "path": req.request_path(),
        "user_intro": user_intro,
    });
    let locale = req.new_cookie("locale".into(), "en-US".into(), None);
    res.set_cookie(&locale);
    res.set_code(StatusCode::CREATED);
    res.set_json_data(data);
    Ok(res.into())
}

pub async fn view(req: Request) -> Result {
    let user_id = req.parse_param("id")?;

    let db_query_start_time = Instant::now();
    let user = User::fetch_by_id(&user_id).await.extract(&req)?;
    let db_query_duration = db_query_start_time.elapsed();

    let mut data = Map::data_entry(user);
    let rego = RegoEngine::shared();
    rego.set_input(json!({
        "method": req.request_method(),
        "path": req.path_segments(),
        "session": req.get_data::<UserSession<Uuid>>(),
    }));
    data.upsert("authorized", rego.eval_allow_query("data.app.user.allow"));

    let mut res = Response::default().context(&req);
    res.record_server_timing("db", None, db_query_duration);
    res.set_json_data(data);
    Ok(res.into())
}
