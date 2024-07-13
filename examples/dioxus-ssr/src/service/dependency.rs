use crate::App;
use zino::prelude::*;

pub async fn list_dependencies() -> Result<Vec<Map>, Error> {
    let resource = "https://libraries.io/api/github/zino-rs/zino/dependencies";
    let mut data = App::fetch_json::<Map>(resource, None)
        .await?
        .remove("dependencies")
        .map(|dependencies| dependencies.into_map_array())
        .unwrap_or_default();
    data.retain(|dep| dep.get_str("kind") != Some("development"));
    data.reverse();
    Ok(data)
}
