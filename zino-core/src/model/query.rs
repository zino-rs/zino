use crate::{extend::JsonObjectExt, request::Validation, Map};
use serde_json::Value;

#[derive(Debug, Clone)]
/// A query type of the model.
pub struct Query {
    // Projection fields.
    fields: Vec<String>,
    // Filters.
    filters: Map,
    // Sort order.
    sort_order: (Option<String>, bool),
    // Limit.
    limit: u64,
    // Offset.
    offset: u64,
}

impl Query {
    /// Creates a new instance.
    #[inline]
    pub fn new(filters: Map) -> Self {
        Self {
            fields: Vec::new(),
            filters,
            sort_order: (None, false),
            limit: 10,
            offset: 0,
        }
    }

    /// Updates the query using the json object and returns the validation result.
    #[must_use]
    pub fn read_map(&mut self, data: &Map) -> Validation {
        let mut validation = Validation::new();
        let filters = &mut self.filters;
        for (key, value) in data {
            match key.as_str() {
                "fields" => {
                    if let Some(fields) = Validation::parse_array(value) {
                        self.fields = fields;
                    }
                }
                "sort" | "sort_by" | "order_by" => {
                    if let Some(sort_by) = Validation::parse_string(value) {
                        self.sort_order.0 = sort_by.into();
                    }
                }
                "ascending" => {
                    if let Some(Ok(ascending)) = Validation::parse_bool(value) {
                        self.sort_order.1 = ascending;
                    }
                }
                "limit" => {
                    if let Some(result) = Validation::parse_u64(value) {
                        match result {
                            Ok(limit) => self.limit = limit,
                            Err(err) => validation.record_fail("limit", err),
                        }
                    }
                }
                "offset" | "skip" => {
                    if let Some(result) = Validation::parse_u64(value) {
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
                                    if let Some(vec) = filters.get_mut(key) {
                                        if let Some(vec) = vec.as_array_mut() {
                                            if index > vec.len() {
                                                vec.resize(index, Value::Null);
                                            }
                                            vec.insert(index, value.to_owned());
                                        }
                                    } else {
                                        let mut vec = Vec::new();
                                        vec.resize(index, Value::Null);
                                        vec.insert(index, value.to_owned());
                                        filters.upsert(key, vec);
                                    }
                                } else if let Some(map) = filters.get_mut(key) {
                                    if let Some(map) = map.as_object_mut() {
                                        map.upsert(path, value.to_owned());
                                    }
                                } else {
                                    let mut map = Map::new();
                                    map.upsert(path, value.to_owned());
                                    filters.upsert(key, map);
                                }
                            }
                        } else if value != "" && value != "all" {
                            filters.insert(key.to_owned(), value.to_owned());
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

    /// Moves all elements from the `filters` into `self`.
    #[inline]
    pub fn append_filters(&mut self, filters: &mut Map) {
        self.filters.append(filters);
    }

    /// Inserts a key-value pair into the query filters.
    #[inline]
    pub fn insert_filter(&mut self, key: impl Into<String>, value: impl Into<Value>) {
        self.filters.upsert(key, value);
    }

    /// Sets the sort order.
    #[inline]
    pub fn set_sort_order(&mut self, sort_by: impl Into<Option<String>>, ascending: bool) {
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

    /// Returns a reference to the filters.
    #[inline]
    pub fn filters(&self) -> &Map {
        &self.filters
    }

    /// Returns the sort order.
    #[inline]
    pub fn sort_order(&self) -> (&str, bool) {
        let sort_order = &self.sort_order;
        (sort_order.0.as_deref().unwrap_or_default(), sort_order.1)
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
        Self {
            fields: Vec::new(),
            filters: Map::new(),
            sort_order: (None, false),
            limit: 10,
            offset: 0,
        }
    }
}
