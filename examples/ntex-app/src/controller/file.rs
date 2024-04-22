use std::time::{Duration, Instant};
use zino::{prelude::*, Cluster, Request, Response, Result};

pub async fn upload(mut req: Request) -> Result {
    let (mut body, files) = req.parse_form_data::<Map>().await?;

    let dir = Cluster::shared_dir("uploads");
    let expires = DateTime::now() + Duration::from_secs(600);
    let mut encryption_duration = Duration::ZERO;
    let mut uploads = Vec::new();
    for mut file in files {
        let mut query = Map::new();
        let access_key_id = AccessKeyId::new();
        query.upsert("access_key_id", access_key_id.to_string());

        let secret_key = SecretAccessKey::new(&access_key_id);
        let security_token =
            SecurityToken::try_new(access_key_id, expires, &secret_key).extract(&req)?;
        query.upsert("security_token", security_token.to_string());

        let encryption_start_time = Instant::now();
        file.encrypt_with(secret_key.as_ref()).extract(&req)?;
        encryption_duration += encryption_start_time.elapsed();

        if let Some(file_name) = file.file_name() {
            file.write(dir.join(file_name)).extract(&req)?;
            query.upsert("file_name", file_name);

            let mut map = Map::new();
            map.upsert("field_name", file.field_name());
            map.upsert("file_name", file_name);
            map.upsert("content_type", file.content_type().map(|m| m.as_ref()));
            map.upsert("url", format!("/file/decrypt?{}", query.to_query_string()));
            uploads.push(map);
        }
    }
    body.upsert("files", uploads);

    let mut res = Response::default().context(&req);
    res.record_server_timing("enc", None, Some(encryption_duration));
    res.set_json_data(Map::data_entry(body));
    Ok(res.into())
}

pub async fn decrypt(req: Request) -> Result {
    let query = req.parse_query::<Map>()?;
    let access_key_id = req.parse_access_key_id()?;
    let secret_key = SecretAccessKey::new(&access_key_id);
    let security_token = req.parse_security_token(secret_key.as_ref())?;
    if security_token.is_expired() {
        reject!(req, forbidden, "the security token has expired");
    }

    let Some(file_name) = query.get_str("file_name") else {
        reject!(req, "file_name", "it should be specified");
    };
    let file_path = Cluster::shared_dir("uploads").join(file_name);

    let mut file = NamedFile::try_from_local(file_path).extract(&req)?;
    let decryption_start_time = Instant::now();
    file.decrypt_with(secret_key).extract(&req)?;

    let decryption_duration = decryption_start_time.elapsed();
    let mut res = Response::default().context(&req);
    res.record_server_timing("dec", None, Some(decryption_duration));
    res.send_file(file);
    Ok(res.into())
}
