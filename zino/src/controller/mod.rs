/// Default controller for the `Model`.
pub trait DefaultController<K> {
    /// A type for the request extractor.
    type Request;

    /// A type for the response result.
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

    /// Fetch models.
    async fn fetch(req: Self::Request) -> Self::Result;

    /// Logically deletes a model.
    async fn soft_delete(req: Self::Request) -> Self::Result;

    /// Locks a model.
    async fn lock(req: Self::Request) -> Self::Result;

    /// Archives a model.
    async fn archive(req: Self::Request) -> Self::Result;

    /// Batch inserts multiple models.
    async fn batch_insert(req: Self::Request) -> Self::Result;

    /// Batch deletes multiple models.
    async fn batch_delete(req: Self::Request) -> Self::Result;

    /// Batch updates multiple models.
    async fn batch_update(req: Self::Request) -> Self::Result;

    /// Imports model data.
    async fn import(req: Self::Request) -> Self::Result;

    /// Exports model data.
    async fn export(req: Self::Request) -> Self::Result;

    /// Gets the tree hierarchy data.
    async fn tree(req: Self::Request) -> Self::Result;

    /// Gets the Avro schema for the model.
    async fn schema(req: Self::Request) -> Self::Result;

    /// Gets the model definition.
    async fn definition(req: Self::Request) -> Self::Result;

    /// Mocks the model data.
    async fn mock(req: Self::Request) -> Self::Result;
}

#[cfg(any(feature = "actix", feature = "axum", feature = "ntex"))]
#[cfg(feature = "orm")]
use zino_core::{
    extension::JsonObjectExt,
    model::{ModelHooks, Mutation, Query},
    orm::{ModelAccessor, ModelHelper},
    request::RequestContext,
    response::{ExtractRejection, Rejection, Response, StatusCode},
    JsonValue, Map,
};

#[cfg(any(feature = "actix", feature = "axum", feature = "ntex"))]
#[cfg(feature = "orm")]
impl<K, M> DefaultController<K> for M
where
    K: Default + std::fmt::Display + PartialEq + std::str::FromStr,
    <K as std::str::FromStr>::Err: std::error::Error + Send + 'static,
    M: ModelAccessor<K>,
{
    type Request = crate::Request;
    type Result = crate::Result;

    async fn new(mut req: Self::Request) -> Self::Result {
        let mut model = Self::new();
        let mut res = req.model_validation(&mut model).await?;
        let extension = req.get_data::<<Self as ModelHooks>::Extension>();
        model
            .before_insert_check(extension.as_ref())
            .await
            .extract(&req)?;

        let validation = model.check_constraints().await.extract(&req)?;
        if !validation.is_success() {
            return Err(Rejection::bad_request(validation).context(&req).into());
        }

        let mut model_snapshot = model.snapshot();
        Self::after_decode(&mut model_snapshot)
            .await
            .extract(&req)?;

        let ctx = model.insert().await.extract(&req)?;
        if let Some(last_insert_id) = ctx.last_insert_id() {
            if model_snapshot.get_i64("id") == Some(0) {
                model_snapshot.upsert("id", last_insert_id);
            }
        }

        Self::translate_model(&mut model_snapshot);
        Self::before_respond(&mut model_snapshot, extension.as_ref())
            .await
            .extract(&req)?;
        res.set_code(StatusCode::CREATED);
        res.set_json_data(Self::data_item(model_snapshot));
        Ok(res.into())
    }

    async fn delete(req: Self::Request) -> Self::Result {
        let id = req.parse_param::<K>("id")?;
        let model = Self::try_get_model(&id).await.extract(&req)?;
        model.delete().await.extract(&req)?;

        let res = Response::default().context(&req);
        Ok(res.into())
    }

    async fn update(mut req: Self::Request) -> Self::Result {
        let id = req.parse_param::<K>("id")?;
        let mut body = req.parse_body().await?;

        let extension = req.get_data::<<Self as ModelHooks>::Extension>();
        let (validation, model) = Self::update_by_id(&id, &mut body, extension)
            .await
            .extract(&req)?;
        let mut res = Response::from(validation).context(&req);
        if res.is_success() {
            let model_filters = model.next_version_filters();
            res.set_json_data(Self::data_item(model_filters));
        }
        Ok(res.into())
    }

    async fn view(req: Self::Request) -> Self::Result {
        let id = req.parse_param::<K>("id")?;
        let extension = req.get_data::<<Self as ModelHooks>::Extension>();
        let mut model = if req.get_query("fetch") == Some("false") {
            Self::find_by_id(&id).await.extract(&req)?
        } else {
            Self::fetch_by_id(&id).await.extract(&req)?
        };
        Self::before_respond(&mut model, extension.as_ref())
            .await
            .extract(&req)?;

        let mut res = Response::default().context(&req);
        res.set_json_data(Self::data_item(model));
        Ok(res.into())
    }

    async fn list(req: Self::Request) -> Self::Result {
        let mut query = match req.get_query("mode") {
            Some("full") => Self::default_query(),
            Some("snapshot") => Self::default_snapshot_query(),
            _ => Self::default_list_query(),
        };
        let mut res = req.query_validation(&mut query)?;
        let extension = req.get_data::<<Self as ModelHooks>::Extension>();
        Self::before_list(&mut query, extension.as_ref())
            .await
            .extract(&req)?;

        let models = if query.populate_enabled() {
            let mut models = Self::fetch(&query).await.extract(&req)?;
            for model in models.iter_mut() {
                Self::before_respond(model, extension.as_ref())
                    .await
                    .extract(&req)?;
            }
            models
        } else {
            let mut models = Self::find(&query).await.extract(&req)?;
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

        let mut data = Self::data_items(models);
        if let Some(page_size) = req.get_query("page_size").and_then(|s| s.parse().ok()) {
            if req.get_query("total_rows").is_none() {
                let total_rows = Self::count(&query).await.extract(&req)?;
                let page_count = total_rows.div_ceil(page_size);
                data.upsert("total_rows", total_rows);
                data.upsert("page_count", page_count);
            }
        }
        res.set_json_data(data);
        Ok(res.into())
    }

    async fn fetch(mut req: Self::Request) -> Self::Result {
        let mut query = Self::default_list_query();
        let mut res = req.query_validation(&mut query)?;
        let mut body = req.parse_body().await?;
        query.append_filters(&mut body);

        let extension = req.get_data::<<Self as ModelHooks>::Extension>();
        Self::before_list(&mut query, extension.as_ref())
            .await
            .extract(&req)?;

        let mut models = Self::fetch(&query).await.extract(&req)?;
        for model in models.iter_mut() {
            Self::before_respond(model, extension.as_ref())
                .await
                .extract(&req)?;
        }

        let mut data = Self::data_items(models);
        if let Some(page_size) = req.get_query("page_size").and_then(|s| s.parse().ok()) {
            if req.get_query("total_rows").is_none() {
                let total_rows = Self::count(&query).await.extract(&req)?;
                let page_count = total_rows.div_ceil(page_size);
                data.upsert("total_rows", total_rows);
                data.upsert("page_count", page_count);
            }
        }
        res.set_json_data(data);
        Ok(res.into())
    }

    async fn soft_delete(req: Self::Request) -> Self::Result {
        let id = req.parse_param::<K>("id")?;
        Self::soft_delete_by_id(&id).await.extract(&req)?;

        let res = Response::default().context(&req);
        Ok(res.into())
    }

    async fn lock(req: Self::Request) -> Self::Result {
        let id = req.parse_param::<K>("id")?;
        Self::lock_by_id(&id).await.extract(&req)?;

        let res = Response::default().context(&req);
        Ok(res.into())
    }

    async fn archive(req: Self::Request) -> Self::Result {
        let id = req.parse_param::<K>("id")?;
        Self::archive_by_id(&id).await.extract(&req)?;

        let res = Response::default().context(&req);
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
                model
                    .before_insert_check(extension.as_ref())
                    .await
                    .extract(&req)?;
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
            let mut res = Response::bad_request();
            res.set_json_data(validations);
            Ok(res.into())
        } else {
            let ctx = Self::insert_many(models).await.extract(&req)?;
            let data = Map::from_entry("rows_affected", ctx.rows_affected());
            let mut res = Response::default().context(&req);
            res.set_json_data(data);
            Ok(res.into())
        }
    }

    async fn batch_delete(mut req: Self::Request) -> Self::Result {
        let data = req.parse_body::<JsonValue>().await?;
        let mut query = if let JsonValue::Object(map) = data {
            Query::new(map)
        } else {
            let primary_key_values = Map::from_entry("$in", data);
            Query::from_entry(Self::PRIMARY_KEY_NAME, primary_key_values)
        };
        let extension = req.get_data::<<Self as ModelHooks>::Extension>();
        Self::before_list(&mut query, extension.as_ref())
            .await
            .extract(&req)?;

        let ctx = Self::delete_many(&query).await.extract(&req)?;
        let data = Map::from_entry("rows_affected", ctx.rows_affected());
        let mut res = Response::default().context(&req);
        res.set_json_data(data);
        Ok(res.into())
    }

    async fn batch_update(mut req: Self::Request) -> Self::Result {
        let data = req.parse_body::<Vec<Map>>().await?;

        // Should use `Self::transaction` when the `Send` bound is resolved
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let mut rows_affected = 0;
        for mut map in data.into_iter() {
            if let Some(id) = map.remove(primary_key_name) {
                let query = Query::from_entry(primary_key_name, id);
                let mut mutation = Mutation::new(map);
                let ctx = Self::update_one(&query, &mut mutation)
                    .await
                    .extract(&req)?;
                rows_affected += ctx.rows_affected().unwrap_or_default();
            }
        }

        let mut res = Response::default().context(&req);
        res.set_json_data(Map::from_entry("rows_affected", rows_affected));
        Ok(res.into())
    }

    async fn import(mut req: Self::Request) -> Self::Result {
        let mut query = Query::new(Map::new());
        let mut res = req.query_validation(&mut query)?;

        let data = req.parse_body::<Vec<Map>>().await?;
        let extension = req.get_data::<<Self as ModelHooks>::Extension>();
        let validate_only = query.validate_only();
        let no_check = query.no_check();
        let limit = query.limit();
        let query_filters = query.filters();
        let (enable_upsert, batch_size) = if query_filters.get_str("upsert") == Some("true") {
            (true, 1)
        } else if validate_only {
            (false, 0)
        } else {
            let batch_size = if let Some(Ok(size)) = query_filters.parse_usize("batch_size") {
                size
            } else {
                1
            };
            (false, batch_size)
        };

        let mut rows_affected = 0;
        let mut validations = Vec::new();
        let mut batch_models = Vec::with_capacity(batch_size);
        for (index, mut map) in data.into_iter().enumerate() {
            if limit > 0 && rows_affected >= limit {
                break;
            }
            if batch_models.len() == batch_size && batch_size > 0 {
                let mut models = Vec::with_capacity(batch_size);
                models.append(&mut batch_models);
                Self::insert_many(models).await.extract(&req)?;
            }
            Self::before_extract()
                .await
                .map_err(|err| Rejection::from_error(err).context(&req))?;
            Self::before_validation(&mut map, extension.as_ref())
                .await
                .extract(&req)?;

            let mut model = Self::new();
            let mut validation = model.read_map(&map);
            if validation.is_success() && !no_check {
                model
                    .before_insert_check(extension.as_ref())
                    .await
                    .extract(&req)?;
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
                if !validate_only {
                    if enable_upsert {
                        model.upsert().await.extract(&req)?;
                    } else if batch_size == 1 {
                        model.insert().await.extract(&req)?;
                    } else {
                        batch_models.push(model);
                    }
                    rows_affected += 1;
                }
            } else {
                let mut map = validation.into_map();
                map.upsert("index", index);

                if validate_only {
                    validations.push(map);
                } else {
                    let mut res = Response::bad_request();
                    res.set_json_data(map);
                    return Ok(res.into());
                }
            }
        }
        if !batch_models.is_empty() {
            Self::insert_many(batch_models).await.extract(&req)?;
        }

        let data = if validations.is_empty() {
            Map::from_entry("rows_affected", rows_affected)
        } else {
            Map::from_entry("validations", validations)
        };
        res.set_json_data(data);
        Ok(res.into())
    }

    async fn export(req: Self::Request) -> Self::Result {
        let mut query = Self::default_query();
        let mut res = req.query_validation(&mut query)?;
        let extension = req.get_data::<<Self as ModelHooks>::Extension>();
        Self::before_list(&mut query, extension.as_ref())
            .await
            .extract(&req)?;

        let mut models = Self::find(&query).await.extract(&req)?;
        let translate_enabled = query.translate_enabled();
        for model in models.iter_mut() {
            Self::after_decode(model).await.extract(&req)?;
            translate_enabled.then(|| Self::translate_model(model));
            Self::before_respond(model, extension.as_ref())
                .await
                .extract(&req)?;
        }

        let format = req.get_query("format").unwrap_or("json");
        match format {
            "csv" => res.set_csv_response(models),
            "jsonlines" => res.set_jsonlines_response(models),
            _ => res.set_json_response(models),
        }
        Ok(res.into())
    }

    async fn tree(req: Self::Request) -> Self::Result {
        let mut query = Self::default_list_query();
        let mut res = req.query_validation(&mut query)?;
        let extension = req.get_data::<<Self as ModelHooks>::Extension>();
        Self::before_list(&mut query, extension.as_ref())
            .await
            .extract(&req)?;

        let parent_id = req.get_query("parent_id").unwrap_or("null");
        query.add_filter("parent_id", parent_id);

        let mut models = if query.populate_enabled() {
            Self::fetch(&query).await.extract(&req)?
        } else {
            let mut models = Self::find(&query).await.extract(&req)?;
            let translate_enabled = query.translate_enabled();
            for model in models.iter_mut() {
                Self::after_decode(model).await.extract(&req)?;
                translate_enabled.then(|| Self::translate_model(model));
            }
            models
        };

        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let values = models
            .iter()
            .filter_map(|model| model.get(primary_key_name).cloned())
            .collect::<Vec<_>>();
        let mut query = Self::default_snapshot_query();
        query.add_filter("parent_id", Map::from_entry("$in", values));
        query.add_filter("status", Map::from_entry("$ne", "Deleted"));
        query.order_desc("parent_id");
        query.order_desc("created_at");
        query.disable_limit();

        let mut children = Self::find::<Map>(&query).await.extract(&req)?;
        let total_rows = children.len();
        for model in models.iter_mut() {
            let model_id = model.get(primary_key_name);

            // Should use `extract_if` when it is stabilized.
            let mut model_children = Vec::new();
            let mut index = 0;
            while index < children.len() {
                if children[index].get("parent_id") == model_id {
                    let child = children.remove(index);
                    model_children.push(child);
                } else {
                    index += 1;
                }
            }
            model.upsert("children", model_children);
        }

        let mut data = Self::data_items(models);
        data.upsert("total_rows", total_rows);
        res.set_json_data(data);
        Ok(res.into())
    }

    async fn schema(req: Self::Request) -> Self::Result {
        let schema = serde_json::to_value(Self::schema()).extract(&req)?;
        let mut res = Response::default().context(&req);
        res.set_json_response(schema);
        Ok(res.into())
    }

    async fn definition(req: Self::Request) -> Self::Result {
        let action = req.get_query("action").unwrap_or("insert");
        let columns = Self::columns();
        let mut definition = Map::new();
        definition.upsert("type", "object");
        if matches!(action, "insert" | "import") {
            let required_fields = columns
                .iter()
                .filter(|&col| {
                    col.is_not_null() && !col.is_primary_key() || col.has_attribute("nonempty")
                })
                .map(|col| col.name())
                .collect::<Vec<_>>();
            definition.upsert("required", required_fields);
        }

        let exclusive_attributes = if action == "update" {
            vec!["read_only", "generated", "reserved"]
        } else {
            vec!["read_only", "generated", "reserved", "auto_initialized"]
        };
        let mut properties = Map::new();
        for col in columns {
            if !col.has_any_attributes(&exclusive_attributes) && col.comment().is_some()
                || matches!(action, "list" | "view" | "export" | "tree")
            {
                properties.upsert(col.name(), col.definition());
            }
        }
        definition.upsert("properties", properties);

        let data = if action == "import" {
            let mut data = Map::new();
            data.upsert("type", "array");
            data.upsert("items", definition);
            data
        } else {
            definition
        };

        let mut res = Response::default().context(&req);
        res.set_json_response(data);
        Ok(res.into())
    }

    async fn mock(req: Self::Request) -> Self::Result {
        let mut query = Query::default();
        let mut res = req.query_validation(&mut query)?;

        let limit = query.limit();
        let validate_only = query.validate_only();
        let mut models = Vec::with_capacity(limit);
        for _ in 0..limit {
            let (validation, model) = Self::mock().await.extract(&req)?;
            if validation.is_success() && !validate_only {
                let mut model_snapshot = model.snapshot();
                let ctx = model.insert().await.extract(&req)?;
                if let Some(last_insert_id) = ctx.last_insert_id() {
                    if model_snapshot.get_i64("id") == Some(0) {
                        model_snapshot.upsert("id", last_insert_id);
                    }
                }
                models.push(model_snapshot);
            } else {
                models.push(model.into_map());
            }
        }

        let data = Self::data_items(models);
        res.set_json_data(data);
        Ok(res.into())
    }
}
