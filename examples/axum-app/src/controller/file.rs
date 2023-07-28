use zino::{prelude::*, Cluster, Request, Response, Result};

pub async fn upload(mut req: Request) -> Result {
    let (mut body, files) = req.parse_form_data::<Map>().await?;

    let dir = Cluster::project_dir().join("assets/uploads");
    let mut uploads = Vec::new();
    for mut file in files {
        let access_key_id = AccessKeyId::new();
        let secret_key = SecretAccessKey::new(&access_key_id);
        file.encrypt_with(secret_key.as_ref()).extract(&req)?;
        if let Some(file_name) = file.file_name() {
            let file_path = dir.join(file_name);
            file.write(file_path).extract(&req)?;

            let url = format!("/file/decrypt?file_name={file_name}&secret_key={secret_key}");
            let mut map = Map::new();
            map.upsert("field_name", file.field_name());
            map.upsert("file_name", file_name);
            map.upsert("content_type", file.content_type().map(|m| m.as_ref()));
            map.upsert("url", url);
            uploads.push(map);
        }
    }
    body.upsert("files", uploads);

    let mut res = Response::default().context(&req);
    res.set_json_data(Map::data_entry(body));
    Ok(res.into())
}

pub async fn decrypt(req: Request) -> Result {
    let query = req.parse_query::<Map>()?;
    let Some(file_name) = query.get_str("file_name") else {
        let err = Error::new("should be specified");
        return Err(Rejection::from_validation_entry("file_name", err).into());
    };
    let file_path = Cluster::project_dir().join(format!("assets/uploads/{file_name}"));

    let mut file = NamedFile::try_from_local(file_path).extract(&req)?;
    if let Some(secret_key) = req.get_query("secret_key") {
        file.decrypt_with(secret_key).extract(&req)?;
    }

    let mut res = Response::default().context(&req);
    res.send_file(file);
    Ok(res.into())
}
