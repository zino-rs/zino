use crate::{
    extension::{JsonObjectExt, JsonValueExt},
    request::Validation,
    JsonValue, Map, SharedString,
};

#[derive(Debug, Clone)]
/// A query type for models.
pub struct Query {
    // Projection fields.
    fields: Vec<String>,
    // Filters.
    filters: Map,
    // Sort order.
    sort_order: (Option<SharedString>, bool),
    // Offset.
    offset: usize,
    // Limit.
    limit: usize,
}

impl Query {
    /// Creates a new instance.
    #[inline]
    pub fn new(filters: impl Into<JsonValue>) -> Self {
        let filters = if let JsonValue::Object(map) = filters.into() {
            map
        } else {
            Map::new()
        };
        Self {
            fields: Vec::new(),
            filters,
            sort_order: (None, false),
            offset: 0,
            limit: 10,
        }
    }

    /// Updates the query using the json object and returns the validation result.
    #[must_use]
    pub fn read_map(&mut self, data: &Map) -> Validation {
        let mut validation = Validation::new();
        let mut pagination_current_page = None;
        let filters = &mut self.filters;
        for (key, value) in data {
            match key.as_str() {
                "fields" | "columns" | "select" => {
                    if let Some(fields) = value.parse_str_array() {
                        self.fields = fields.into_iter().map(|s| s.to_owned()).collect();
                    }
                }
                "sort" | "sort_by" | "order" | "order_by" => {
                    if let Some(sort_by) = value.parse_string() {
                        self.sort_order.0 = Some(sort_by.into_owned().into());
                    }
                }
                "ascending" => {
                    if let Some(Ok(ascending)) = value.parse_bool() {
                        self.sort_order.1 = ascending;
                    }
                }
                "offset" | "skip" => {
                    if let Some(result) = value.parse_usize() {
                        match result {
                            Ok(offset) => self.offset = offset,
                            Err(err) => validation.record_fail("offset", err),
                        }
                    }
                }
                "limit" | "page_size" => {
                    if let Some(result) = value.parse_usize() {
                        match result {
                            Ok(limit) => self.limit = limit,
                            Err(err) => validation.record_fail("limit", err),
                        }
                    }
                }
                "current_page" => {
                    if let Some(result) = value.parse_usize() {
                        match result {
                            Ok(current_page) => pagination_current_page = Some(current_page),
                            Err(err) => validation.record_fail("current_page", err),
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
                                                vec.resize(index, JsonValue::Null);
                                            }
                                            vec.insert(index, value.to_owned());
                                        }
                                    } else {
                                        let mut vec = Vec::with_capacity(index);
                                        vec.resize(index, JsonValue::Null);
                                        vec.push(value.to_owned());
                                        filters.upsert(key, vec);
                                    }
                                } else if let Some(map) = filters.get_mut(key) {
                                    if let Some(map) = map.as_object_mut() {
                                        map.upsert(path, value.to_owned());
                                    }
                                } else {
                                    filters.upsert(key, Map::from_entry(path, value.to_owned()));
                                }
                            }
                        } else if value != "" && value != "all" {
                            filters.insert(key.to_owned(), value.to_owned());
                        }
                    }
                }
            }
        }
        if let Some(current_page) = pagination_current_page {
            self.offset = self.limit * current_page.saturating_sub(1);
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
                    .any(|key| field == key || field.starts_with(&format!("{key}:")))
            })
        }
    }

    /// Removes the projection fields in the deny list.
    #[inline]
    pub fn deny_fields(&mut self, fields: &[&str]) {
        self.fields.retain(|field| {
            !fields
                .iter()
                .any(|key| field == key || field.starts_with(&format!("{key}:")))
        })
    }

    /// Adds a key-value pair to the query filters.
    #[inline]
    pub fn add_filter(&mut self, key: impl Into<String>, value: impl Into<JsonValue>) {
        self.filters.upsert(key, value);
    }

    /// Moves all elements from the `filters` into `self`.
    #[inline]
    pub fn append_filters(&mut self, filters: &mut Map) {
        self.filters.append(filters);
    }

    /// Sets the sort order.
    #[inline]
    pub fn set_sort_order(&mut self, sort_by: impl Into<SharedString>, ascending: bool) {
        self.sort_order = (Some(sort_by.into()), ascending);
    }

    /// Sets the query offset.
    #[inline]
    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    /// Sets the query limit.
    #[inline]
    pub fn set_limit(&mut self, limit: usize) {
        self.limit = limit;
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

    /// Returns the query offset.
    #[inline]
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Returns the query limit.
    #[inline]
    pub fn limit(&self) -> usize {
        self.limit
    }
}

impl Default for Query {
    #[inline]
    fn default() -> Self {
        Self {
            fields: Vec::new(),
            filters: Map::new(),
            sort_order: (None, false),
            offset: 0,
            limit: 10,
        }
    }
}
