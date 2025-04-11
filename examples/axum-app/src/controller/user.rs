use crate::model::{User, UserColumn::*};
use std::time::Instant;
use zino::{Request, Response, Result, prelude::*};

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
        "method": req.request_method().as_ref(),
        "path": req.request_path(),
        "user_intro": user_intro,
    });
    let locale = req.new_cookie("locale".into(), "en-US".into(), None);
    res.set_cookie(&locale);
    res.set_json_data(data);
    Ok(res.into())
}

pub async fn view(req: Request) -> Result {
    let user_id = req.parse_param("id")?;

    let db_query_start_time = Instant::now();
    let user = User::fetch_by_id(&user_id).await.extract(&req)?;
    let db_query_duration = db_query_start_time.elapsed();

    let data = Map::data_entry(user);
    let mut res = Response::default().context(&req);
    res.record_server_timing("db", None, db_query_duration);
    res.set_json_data(data);
    Ok(res.into())
}

pub async fn stats(req: Request) -> Result {
    let query = QueryBuilder::new()
        .aggregate(Aggregation::Count(Id, false), Some("num_users"))
        .aggregate(Aggregation::Sum(LoginCount), Some("total_login"))
        .aggregate(Aggregation::Avg(LoginCount), None)
        .and_not_in(Status, ["Deleted", "Locked"])
        .and_ge(DerivedColumn::year(CreatedAt), Date::today().year())
        .group_by(CurrentLoginIp, None)
        .group_by(DerivedColumn::date(CurrentLoginAt), Some("login_date"))
        .having_ge(Aggregation::Avg(LoginCount), 10)
        .order_desc(DerivedColumn::alias("total_login"))
        .limit(10)
        .build();
    let items = User::aggregate::<Map>(&query).await.extract(&req)?;

    let mut res = Response::default().context(&req);
    res.set_json_data(User::data_items(items));
    Ok(res.into())
}
