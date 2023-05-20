use zino::prelude::*;
use zino_model::User;

pub(crate) async fn update(user_id: Uuid, body: Map) -> Result<(Validation, Map), Error> {
    let user_id = user_id.to_string();
    let mut user = User::try_get_model(&user_id).await?;
    let validation = user.read_map(&body);
    if !validation.is_success() {
        return Ok((validation, Map::new()));
    }

    let query = user.current_version_query();
    let mutation = user.next_version_mutation(body);
    User::update_one(&query, &mutation).await?;

    let user_info = user.next_version_filters();
    Ok((validation, user_info))
}
