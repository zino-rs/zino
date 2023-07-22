/// Default controller for the `Model`.
pub trait DefaultController<K, U = K> {
    /// The request extractor.
    type Request;
    /// The response result.
    type Result;

    /// Creates a new model.
    async fn new(req: Self::Request) -> Self::Result;

    /// Deletes a model.
    async fn delete(req: Self::Request) -> Self::Result;

    /// Updates a model.
    async fn update(req: Self::Request) -> Self::Result;

    /// Views a model.
    async fn view(req: Self::Request) -> Self::Result;

    /// Lists models.
    async fn list(req: Self::Request) -> Self::Result;

    /// Batch inserts multiple models.
    async fn batch_insert(req: Self::Request) -> Self::Result;

    /// Batch deletes multiple models.
    async fn batch_delete(req: Self::Request) -> Self::Result;

    /// Imports model data.
    async fn import(req: Self::Request) -> Self::Result;

    /// Exports model data.
    async fn export(req: Self::Request) -> Self::Result;
}

#[cfg(any(feature = "actix", feature = "axum"))]
#[cfg(feature = "orm")]
use zino_core::{
    database::{ModelAccessor, ModelHelper},
    error::Error,
    extension::{JsonObjectExt, JsonValueExt},
    model::{ModelHooks, Query},
    request::RequestContext,
    response::{ExtractRejection, Rejection, StatusCode},
    JsonValue, Map,
};

#[cfg(any(feature = "actix", feature = "axum"))]
#[cfg(feature = "orm")]
impl<K, U, M: ModelAccessor<K, U>> DefaultController<K, U> for M
where
    K: Default + std::fmt::Display + PartialEq + serde::de::DeserializeOwned,
    U: Default + std::fmt::Display + PartialEq,
{
    type Request = crate::Request;
    type Result = crate::Result;

    async fn new(mut req: Self::Request) -> Self::Result {
        let mut model = Self::new();
        let mut res = req.model_validation(&mut model).await?;
        let validation = model.check_constraints().await.extract(&req)?;
        if !validation.is_success() {
            return Err(Rejection::bad_request(validation).context(&req).into());
        }

        let mut model_snapshot = model.snapshot();
        Self::after_decode(&mut model_snapshot)
            .await
            .extract(&req)?;
        model.insert().await.extract(&req)?;

        let extension = req.get_data::<<Self as ModelHooks>::Extension>();
        Self::translate_model(&mut model_snapshot);
        Self::before_respond(&mut model_snapshot, extension.as_ref())
            .await
            .extract(&req)?;

        let data = Map::data_entry(model_snapshot);
        res.set_code(StatusCode::CREATED);
        res.set_data(&data);
        Ok(res.into())
    }

    async fn delete(req: Self::Request) -> Self::Result {
        let id = req.parse_param::<K>("id")?;
        Self::soft_delete_by_id(&id).await.extract(&req)?;

        let res = crate::Response::default().context(&req);
        Ok(res.into())
    }

    async fn update(mut req: Self::Request) -> Self::Result {
        let id = req.parse_param::<K>("id")?;
        let mut body = req.parse_body().await?;

        let extension = req.get_data::<<Self as ModelHooks>::Extension>();
        let (validation, model) = Self::update_by_id(&id, &mut body, extension)
            .await
            .extract(&req)?;
        let mut res = crate::Response::from(validation).context(&req);
        if res.is_success() {
            let model_filters = model.next_version_filters();
            let data = Map::data_entry(model_filters);
            res.set_data(&data);
        }
        Ok(res.into())
    }

    async fn view(req: Self::Request) -> Self::Result {
        let id = req.parse_param::<K>("id")?;
        let model = Self::fetch_by_id(&id).await.extract(&req)?;
        let data = Map::data_entry(model);
        let mut res = crate::Response::default().context(&req);
        res.set_data(&data);
        Ok(res.into())
    }

    async fn list(req: Self::Request) -> Self::Result {
        let mut query = Self::default_list_query();
        let extension = req.get_data::<<Self as ModelHooks>::Extension>();
        Self::before_list(&mut query, extension.as_ref())
            .await
            .extract(&req)?;

        let mut res = req.query_validation(&mut query)?;
        let models = if query.populate_enabled() {
            let mut models = Self::fetch(&mut query).await.extract(&req)?;
            for model in models.iter_mut() {
                Self::before_respond(model, extension.as_ref())
                    .await
                    .extract(&req)?;
            }
            models
        } else {
            let mut models = Self::find(&mut query).await.extract(&req)?;
            let translate_enabled = query.translate_enabled();
            for model in models.iter_mut() {
                Self::after_decode(model).await.extract(&req)?;
                translate_enabled.then(|| Self::translate_model(model));
                Self::before_respond(model, extension.as_ref())
                    .await
                    .extract(&req)?;
            }
            models
        };

        let mut data = Map::data_entries(models);
        if req.get_query("page_size").is_some() {
            let total_rows = Self::count(&mut query).await.extract(&req)?;
            data.upsert("total_rows", total_rows);
        }
        res.set_data(&data);
        Ok(res.into())
    }

    async fn batch_insert(mut req: Self::Request) -> Self::Result {
        let data = req.parse_body::<Vec<Map>>().await?;
        let extension = req.get_data::<<Self as ModelHooks>::Extension>();
        let mut models = Vec::with_capacity(data.len());
        let mut validations = Vec::new();
        for (index, mut map) in data.into_iter().enumerate() {
            Self::before_extract()
                .await
                .map_err(|err| Rejection::from_error(err).context(&req))?;
            Self::before_validation(&mut map, extension.as_ref())
                .await
                .extract(&req)?;

            let mut model = Self::new();
            let mut validation = model.read_map(&map);
            if validation.is_success() {
                validation = model.check_constraints().await.extract(&req)?;
            }
            if validation.is_success() {
                model.after_validation(&mut map).await.extract(&req)?;
                if let Some(ref extension) = extension {
                    model
                        .after_extract(extension.clone())
                        .await
                        .map_err(|err| Rejection::from_error(err).context(&req))?;
                }
                models.push(model);
            } else {
                let mut map = validation.into_map();
                map.upsert("index", index);
                validations.push(map);
            }
        }
        if !validations.is_empty() {
            let mut res = crate::Response::new(StatusCode::BAD_REQUEST);
            res.set_data(&validations);
            Ok(res.into())
        } else {
            let rows_affected = Self::insert_many(models).await.extract(&req)?;
            let data = Map::from_entry("rows_affected", rows_affected);
            let mut res = crate::Response::default().context(&req);
            res.set_code(StatusCode::CREATED);
            res.set_data(&data);
            Ok(res.into())
        }
    }

    async fn batch_delete(mut req: Self::Request) -> Self::Result {
        let data = req.parse_body::<Vec<JsonValue>>().await?;
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let primary_key_values = Map::from_entry("$in", data);
        let filters = Map::from_entry(primary_key_name, primary_key_values);
        let mut query = Query::new(filters);

        let rows_affected = Self::delete_many(&mut query).await.extract(&req)?;
        let data = Map::from_entry("rows_affected", rows_affected);
        let mut res = crate::Response::default().context(&req);
        res.set_data(&data);
        Ok(res.into())
    }

    async fn import(mut req: Self::Request) -> Self::Result {
        let is_upsert_mode = req.get_query("mode").is_some_and(|s| s == "upsert");
        let data = req.parse_body::<Vec<Map>>().await?;
        let extension = req.get_data::<<Self as ModelHooks>::Extension>();
        let mut rows_affected = 0;
        for (index, mut map) in data.into_iter().enumerate() {
            Self::before_extract()
                .await
                .map_err(|err| Rejection::from_error(err).context(&req))?;
            Self::before_validation(&mut map, extension.as_ref())
                .await
                .extract(&req)?;

            let mut model = Self::new();
            let mut validation = model.read_map(&map);
            if validation.is_success() {
                validation = model.check_constraints().await.extract(&req)?;
            }
            if validation.is_success() {
                model.after_validation(&mut map).await.extract(&req)?;
                if let Some(ref extension) = extension {
                    model
                        .after_extract(extension.clone())
                        .await
                        .map_err(|err| Rejection::from_error(err).context(&req))?;
                }
                if is_upsert_mode {
                    model.upsert().await.extract(&req)?;
                } else {
                    model.insert().await.extract(&req)?;
                }
                rows_affected += 1;
            } else {
                let mut map = validation.into_map();
                map.upsert("index", index);

                let mut res = crate::Response::new(StatusCode::BAD_REQUEST);
                res.set_data(&map);
                return Ok(res.into());
            }
        }

        let data = Map::from_entry("rows_affected", rows_affected);
        let mut res = crate::Response::default().context(&req);
        res.set_data(&data);
        Ok(res.into())
    }

    async fn export(req: Self::Request) -> Self::Result {
        let mut query = Self::default_query();
        let extension = req.get_data::<<Self as ModelHooks>::Extension>();
        Self::before_list(&mut query, extension.as_ref())
            .await
            .extract(&req)?;

        let mut res = req.query_validation(&mut query)?;
        let mut models = Self::find(&mut query).await.extract(&req)?;
        let translate_enabled = query.translate_enabled();
        for model in models.iter_mut() {
            Self::after_decode(model).await.extract(&req)?;
            translate_enabled.then(|| Self::translate_model(model));
            Self::before_respond(model, extension.as_ref())
                .await
                .extract(&req)?;
        }

        let data = Map::data_entries(models);
        res.set_data(&data);

        let format = req.get_query("format").unwrap_or("json");
        match format {
            "csv" => {
                res.set_content_type("text/csv; charset=utf-8");
                res.set_data_transformer(|data| {
                    if let Some(value) = data.pointer("/entries") {
                        value.to_csv(Vec::new()).map_err(Error::from)
                    } else {
                        Ok(Vec::new())
                    }
                });
            }
            "jsonlines" => {
                res.set_content_type("application/jsonlines; charset=utf-8");
                res.set_data_transformer(|data| {
                    if let Some(value) = data.pointer("/entries") {
                        value.to_jsonlines(Vec::new()).map_err(Error::from)
                    } else {
                        Ok(Vec::new())
                    }
                });
            }
            "msgpack" => {
                res.set_content_type("application/msgpack");
                res.set_data_transformer(|data| {
                    if let Some(value) = data.pointer("/entries") {
                        value.to_msgpack(Vec::new()).map_err(Error::from)
                    } else {
                        Ok(Vec::new())
                    }
                });
            }
            _ => {
                res.set_data_transformer(|data| {
                    serde_json::to_vec(&data.pointer("/entries")).map_err(Error::from)
                });
            }
        }
        Ok(res.into())
    }
}
