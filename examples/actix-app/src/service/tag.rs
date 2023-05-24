use zino::prelude::*;
use zino_model::Tag;

pub async fn find(query: &Query) -> Result<Vec<Map>, Error> {
    let mut tags = Tag::find(query).await?;
    let mut query = Tag::default_snapshot_query();
    Tag::find_related(&mut query, &mut tags, ["parent_id"]).await?;
    Ok(tags)
}

pub async fn find_by_id(tag_id: &Uuid) -> Result<Map, Error> {
    let mut tag: Map = Tag::find_by_id(tag_id)
        .await?
        .ok_or_else(|| Error::new(format!("cannot find the tag `{tag_id}`")))?;
    let mut query = Tag::default_snapshot_query();
    Tag::find_related_one(&mut query, &mut tag, ["parent_id"]).await?;
    Ok(tag)
}
