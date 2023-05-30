use zino::{prelude::*, Request, Response, Result};
use zino_model::Tag;

pub async fn new(mut req: Request) -> Result {
    let mut tag = Tag::new();
    let mut res: Response = req.model_validation(&mut tag).await?;

    let data = Map::data_entry(tag.snapshot());
    tag.upsert().await.extract(&req)?;
    res.set_data(&data);
    Ok(res.into())
}

pub async fn update(mut req: Request) -> Result {
    let tag_id: Uuid = req.parse_param("id")?;
    let body: Map = req.parse_body().await?;
    let (validation, tag) = Tag::update_by_id(&tag_id, body).await.extract(&req)?;
    let data = Map::data_entry(tag.next_version_filters());
    let mut res = Response::from(validation).context(&req);
    res.set_data(&data);
    Ok(res.into())
}

pub async fn list(req: Request) -> Result {
    let mut query = Tag::default_list_query();
    let mut res: Response = req.query_validation(&mut query)?;
    let tags = Tag::fetch(&query).await.extract(&req)?;
    let data = Map::data_entries(tags);
    res.set_data(&data);
    Ok(res.into())
}

pub async fn view(req: Request) -> Result {
    let tag_id: Uuid = req.parse_param("id")?;
    let tag = Tag::fetch_by_id(&tag_id).await.extract(&req)?;

    let data = Map::data_entry(tag);
    let mut res = Response::default().context(&req);
    res.set_data(&data);
    Ok(res.into())
}

pub async fn delete(req: Request) -> Result {
    let id: Uuid = req.parse_param("id")?;
    Tag::soft_delete_by_id(&id).await.extract(&req)?;

    let res = Response::default().context(&req);
    Ok(res.into())
}
