/// Default controller for the `Model`.
pub trait DefaultController<T, U = T> {
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
    database::ModelAccessor,
    error::Error,
    extension::{JsonObjectExt, JsonValueExt},
    model::Query,
    request::RequestContext,
    response::{ExtractRejection, Rejection, StatusCode},
    JsonValue, Map,
};

#[cfg(any(feature = "actix", feature = "axum"))]
#[cfg(feature = "orm")]
impl<T, U, M: ModelAccessor<T, U>> DefaultController<T, U> for M
where
    T: Default + std::fmt::Display + PartialEq + serde::de::DeserializeOwned,
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

        let data = Map::data_entry(model.snapshot());
        model.insert().await.extract(&req)?;
        res.set_code(StatusCode::CREATED);
        res.set_data(&data);
        Ok(res.into())
    }

    async fn delete(req: Self::Request) -> Self::Result {
        let id = req.parse_param::<T>("id")?;
        Self::soft_delete_by_id(&id).await.extract(&req)?;

        let res = crate::Response::default().context(&req);
        Ok(res.into())
    }

    async fn update(mut req: Self::Request) -> Self::Result {
        let id = req.parse_param::<T>("id")?;
        let body = req.parse_body().await?;
        let (validation, model) = Self::update_by_id(&id, body).await.extract(&req)?;
        let mut res = crate::Response::from(validation).context(&req);
        if res.is_success() {
            let data = Map::data_entry(model.next_version_filters());
            res.set_data(&data);
        }
        Ok(res.into())
    }

    async fn view(req: Self::Request) -> Self::Result {
        let id = req.parse_param::<T>("id")?;
        let model = Self::fetch_by_id(&id).await.extract(&req)?;

        let data = Map::data_entry(model);
        let mut res = crate::Response::default().context(&req);
        res.set_data(&data);
        Ok(res.into())
    }

    async fn list(req: Self::Request) -> Self::Result {
        let mut query = Self::default_list_query();
        let mut res = req.query_validation(&mut query)?;
        let populate = query
            .filters()
            .parse_bool("populate")
            .and_then(|r| r.ok())
            .unwrap_or(false);
        let models = if populate {
            Self::fetch(&query).await.extract(&req)?
        } else {
            Self::find(&query).await.extract(&req)?
        };

        let mut data = Map::data_entries(models);
        if req.get_query("page_size").is_some() {
            let total_rows = Self::count(&query).await.extract(&req)?;
            data.upsert("total_rows", total_rows);
        }
        res.set_data(&data);
        Ok(res.into())
    }

    async fn batch_insert(mut req: Self::Request) -> Self::Result {
        let data = req.parse_body::<Vec<Map>>().await?;
        let mut models = Vec::with_capacity(data.len());
        let mut validations = Vec::new();
        for (index, mut map) in data.into_iter().enumerate() {
            Self::before_validation(&mut map).await.extract(&req)?;

            let mut model = Self::new();
            let mut validation = model.read_map(&map);
            if validation.is_success() {
                validation = model.check_constraints().await.extract(&req)?;
            }
            if validation.is_success() {
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
        let query = Query::new(Map::from_entry(primary_key_name, primary_key_values));

        let rows_affected = Self::delete_many(&query).await.extract(&req)?;
        let data = Map::from_entry("rows_affected", rows_affected);
        let mut res = crate::Response::default().context(&req);
        res.set_data(&data);
        Ok(res.into())
    }

    async fn import(mut req: Self::Request) -> Self::Result {
        let is_upsert_mode = req
            .get_query("mode")
            .map(|s| s == "upsert")
            .unwrap_or_default();
        let data = req.parse_body::<Vec<Map>>().await?;
        let mut rows_affected = 0;
        for (index, mut map) in data.into_iter().enumerate() {
            Self::before_validation(&mut map).await.extract(&req)?;

            let mut model = Self::new();
            let mut validation = model.read_map(&map);
            if validation.is_success() {
                validation = model.check_constraints().await.extract(&req)?;
            }
            if validation.is_success() {
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
        let mut res = req.query_validation(&mut query)?;
        let models = Self::find(&query).await.extract(&req)?;
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
