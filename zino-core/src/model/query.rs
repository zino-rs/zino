use crate::{extend::JsonObjectExt, request::Validation, Map, SharedString};
use serde_json::Value;

#[derive(Debug, Clone)]
/// A query type of the model.
pub struct Query {
    // Projection fields.
    fields: Vec<String>,
    // Filter.
    filter: Map,
    // Sort order.
    sort_order: (SharedString, bool),
    // Limit.
    limit: u64,
    // Offset.
    offset: u64,
}

impl Query {
    /// Creates a new instance.
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            filter: Map::new(),
            sort_order: ("".into(), false),
            limit: 10,
            offset: 0,
        }
    }

    /// Updates the query using the json object and returns the validation result.
    #[must_use]
    pub fn read_map(&mut self, data: Map) -> Validation {
        let mut validation = Validation::new();
        let filter = &mut self.filter;
        for (key, value) in data.into_iter() {
            match key.as_str() {
                "fields" => {
                    if let Some(fields) = Validation::parse_array(&value) {
                        self.fields = fields;
                    }
                }
                "sort_by" | "order_by" => {
                    if let Some(sort_by) = Validation::parse_string(&value) {
                        self.sort_order.0 = sort_by.into();
                    }
                }
                "ascending" => {
                    if let Some(Ok(ascending)) = Validation::parse_bool(&value) {
                        self.sort_order.1 = ascending;
                    }
                }
                "limit" => {
                    if let Some(result) = Validation::parse_u64(&value) {
                        match result {
                            Ok(limit) => self.limit = limit,
                            Err(err) => validation.record_fail("limit", err),
                        }
                    }
                }
                "offset" | "skip" => {
                    if let Some(result) = Validation::parse_u64(&value) {
                        match result {
                            Ok(offset) => self.offset = offset,
                            Err(err) => validation.record_fail("offset", err),
                        }
                    }
                }
                "timestamp" | "nonce" | "signature" => (),
                _ => {
                    if !key.starts_with('$') {
                        if key.contains('.') {
                            if let Some((key, path)) = key.split_once('.') {
                                if let Ok(index) = path.parse::<usize>() {
                                    if let Some(vec) = filter.get_mut(key) {
                                        if let Some(vec) = vec.as_array_mut() {
                                            if index > vec.len() {
                                                vec.resize(index, Value::Null);
                                            }
                                            vec.insert(index, value);
                                        }
                                    } else {
                                        let mut vec = Vec::new();
                                        vec.resize(index, Value::Null);
                                        vec.insert(index, value);
                                        filter.upsert(key, vec);
                                    }
                                } else if let Some(map) = filter.get_mut(key) {
                                    if let Some(map) = map.as_object_mut() {
                                        map.upsert(path, value);
                                    }
                                } else {
                                    let mut map = Map::new();
                                    map.upsert(path, value);
                                    filter.upsert(key, map);
                                }
                            }
                        } else if value != "" && value != "all" {
                            filter.insert(key, value);
                        }
                    }
                }
            }
        }
        validation
    }

    /// Retains the projection fields in the allow list.
    /// If the projection fields are empty, it will be set to the list.
    #[inline]
    pub fn allow_fields(&mut self, fields: &[&str]) {
        if self.fields.is_empty() {
            self.fields = fields.iter().map(|&key| key.to_owned()).collect::<Vec<_>>();
        } else {
            self.fields.retain(|field| {
                fields
                    .iter()
                    .any(|key| field == key || field.ends_with(&format!(" {key}")))
            })
        }
    }

    /// Removes the projection fields in the deny list.
    #[inline]
    pub fn deny_fields(&mut self, fields: &[&str]) {
        self.fields.retain(|field| {
            !fields
                .iter()
                .any(|key| field == key || field.ends_with(&format!(" {key}")))
        })
    }

    /// Appends to the query filter.
    #[inline]
    pub fn append_filter(&mut self, filter: &mut Map) {
        self.filter.append(filter);
    }

    /// Inserts a key-value pair into the query filter.
    #[inline]
    pub fn insert_filter(&mut self, key: impl Into<String>, value: impl Into<Value>) {
        self.filter.upsert(key, value);
    }

    /// Sets the sort order.
    #[inline]
    pub fn set_sort_order(&mut self, sort_by: impl Into<SharedString>, ascending: bool) {
        self.sort_order = (sort_by.into(), ascending);
    }

    /// Sets the query limit.
    #[inline]
    pub fn set_limit(&mut self, limit: u64) {
        self.limit = limit;
    }

    /// Sets the query offset.
    #[inline]
    pub fn set_offset(&mut self, offset: u64) {
        self.offset = offset;
    }

    /// Returns a reference to the projection fields.
    #[inline]
    pub fn fields(&self) -> &[String] {
        self.fields.as_slice()
    }

    /// Returns a reference to the filter.
    #[inline]
    pub fn filter(&self) -> &Map {
        &self.filter
    }

    /// Returns the sort order.
    #[inline]
    pub fn sort_order(&self) -> (&str, bool) {
        let sort_order = &self.sort_order;
        (sort_order.0.as_ref(), sort_order.1)
    }

    /// Returns the query limit.
    #[inline]
    pub fn limit(&self) -> u64 {
        self.limit
    }

    /// Returns the query offset.
    #[inline]
    pub fn offset(&self) -> u64 {
        self.offset
    }
}

impl Default for Query {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
