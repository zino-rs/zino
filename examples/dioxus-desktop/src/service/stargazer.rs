use crate::App;
use serde_json::json;
use zino::prelude::*;

pub async fn list_stargazers(per_page: u8, page: u32) -> Result<Vec<Map>, Error> {
    let resource = "https://api.github.com/repos/photino/zino/stargazers";
    let options = json!({
        "query": {
            "per_page": per_page,
            "page": page,
        },
    });
    App::fetch_json(resource, options.into_map_opt().as_ref()).await
}
