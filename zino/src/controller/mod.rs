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

    /// Imports model data.
    async fn import(req: Self::Request) -> Self::Result;

    /// Exports model data.
    async fn export(req: Self::Request) -> Self::Result;
}

#[cfg(any(feature = "actix", feature = "axum"))]
#[cfg(feature = "orm")]
use zino_core::{
    database::ModelAccessor,
    extension::JsonObjectExt,
    request::RequestContext,
    response::{ExtractRejection, Rejection},
    Map,
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
        res.set_code(zino_core::response::StatusCode::CREATED);
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
        let data = Map::data_entries(models);
        res.set_data(&data);
        Ok(res.into())
    }

    async fn import(mut req: Self::Request) -> Self::Result {
        use zino_core::response::StatusCode;

        let data = req.parse_body::<Vec<Map>>().await?;
        let mut models = Vec::with_capacity(data.len());
        let mut validations = Vec::new();
        for (index, map) in data.iter().enumerate() {
            let mut model = Self::new();
            let mut validation = model.read_map(map);
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

    async fn export(req: Self::Request) -> Self::Result {
        let mut query = Self::default_query();
        let mut res = req.query_validation(&mut query)?;
        let models = Self::find(&query).await.extract(&req)?;
        let data = Map::data_entries(models);
        res.set_data(&data);
        res.set_json_pointer("/entries");
        Ok(res.into())
    }
}
