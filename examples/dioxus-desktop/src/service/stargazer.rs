use crate::App;
use serde_json::json;
use zino::prelude::*;

pub async fn list_stargazers(per_page: u8, page: usize) -> Result<Vec<Map>, Error> {
    let resource = "https://api.github.com/repos/photino/zino/stargazers";
    let options = json!({
        "query": {
            "per_page": per_page,
            "page": page,
        },
        "headers": {
            "accept": "application/vnd.github.star+json",
        }
    });
    let mut data: Vec<Map> = App::fetch_json(resource, options.as_object()).await?;
    for d in data.iter_mut() {
        if let Some(user) = d.remove("user") && let Some(mut user) = user.into_map_opt() {
            d.append(&mut user);
        }
    }
    Ok(data)
}
