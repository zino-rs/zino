use zino::prelude::*;
use zino_model::{Tag, User};

pub async fn find(query: &Query) -> Result<Vec<Map>, Error> {
    let mut users = User::find(query).await?;
    let mut query = Tag::default_snapshot_query();
    Tag::find_related(&mut query, &mut users, ["tags"]).await?;
    Ok(users)
}

pub async fn find_by_id(user_id: &Uuid) -> Result<Map, Error> {
    let mut user: Map = User::find_by_id(user_id)
        .await?
        .ok_or_else(|| Error::new(format!("cannot find the user `{user_id}`")))?;
    let mut query = Tag::default_snapshot_query();
    Tag::find_related_one(&mut query, &mut user, ["tags"]).await?;
    Ok(user)
}
