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
    sort_order: Vec<(SharedString, bool)>,
    // Offset.
    offset: usize,
    // Limit.
    limit: usize,
}

impl Query {
    /// Creates a new instance.
    #[inline]
    pub fn new(filters: impl Into<JsonValue>) -> Self {
        Self {
            fields: Vec::new(),
            filters: filters.into().into_map_opt().unwrap_or_default(),
            sort_order: Vec::new(),
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
                "fields" | "columns" => {
                    if let Some(fields) = value.parse_str_array() {
                        self.fields = fields.into_iter().map(|s| s.to_owned()).collect();
                    }
                }
                "order_by" | "sort_by" => {
                    if let Some(sort_order) = value.parse_str_array() {
                        self.sort_order = sort_order
                            .into_iter()
                            .map(|s| {
                                if let Some(sort) = s.strip_suffix("|asc") {
                                    (sort.to_owned().into(), false)
                                } else if let Some(sort) = s.strip_suffix("|desc") {
                                    (sort.to_owned().into(), true)
                                } else {
                                    (s.to_owned().into(), true)
                                }
                            })
                            .collect::<Vec<_>>();
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
                "populate" | "translate" => {
                    if let Some(result) = value.parse_bool() {
                        match result {
                            Ok(flag) => {
                                filters.upsert(key, flag);
                            }
                            Err(err) => validation.record_fail(key.to_owned(), err),
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
    pub fn set_sort_order(&mut self, field: impl Into<SharedString>, descending: bool) {
        let field = field.into();
        self.sort_order.retain(|(s, _)| s != &field);
        self.sort_order.push((field, descending));
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
    pub fn sort_order(&self) -> &[(SharedString, bool)] {
        &self.sort_order
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

    /// Returns `true` if the `flag` has been enabled.
    #[inline]
    pub fn enabled(&self, flag: &str) -> bool {
        self.filters.get_bool(flag).is_some_and(|b| b)
    }

    /// Returns `true` if the `populate` flag has been enabled.
    #[inline]
    pub fn populate_enabled(&self) -> bool {
        self.enabled("populate")
    }

    /// Returns `true` if the `translate` flag has been enabled.
    #[inline]
    pub fn translate_enabled(&self) -> bool {
        self.enabled("translate")
    }
}

impl Default for Query {
    #[inline]
    fn default() -> Self {
        Self {
            fields: Vec::new(),
            filters: Map::new(),
            sort_order: Vec::new(),
            offset: 0,
            limit: 10,
        }
    }
}

/// A builder type for model queries.
#[derive(Debug, Default)]
pub struct QueryBuilder {
    // Projection fields.
    fields: Vec<String>,
    // Filters.
    filters: Map,
    // Sort order.
    sort_order: Vec<(SharedString, bool)>,
    // Offset.
    offset: usize,
    // Limit.
    limit: usize,
}

impl QueryBuilder {
    /// Creates a new instance.
    #[inline]
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            filters: Map::new(),
            sort_order: Vec::new(),
            offset: 0,
            limit: usize::MAX,
        }
    }

    /// Adds a filter with the condition for equal parts.
    #[inline]
    pub fn and_eq<S, T>(&mut self, field: S, value: T)
    where
        S: Into<String>,
        T: Into<JsonValue>,
    {
        self.filters.upsert(field.into(), value);
    }

    /// Adds a filter with the condition for non-equal parts.
    #[inline]
    pub fn and_ne<S, T>(&mut self, field: S, value: T)
    where
        S: Into<String>,
        T: Into<JsonValue>,
    {
        self.filters
            .upsert(field.into(), Map::from_entry("$ne", value));
    }

    /// Adds a filter with the condition for a field less than the value.
    #[inline]
    pub fn and_lt<S, T>(&mut self, field: S, value: T)
    where
        S: Into<String>,
        T: Into<JsonValue>,
    {
        self.filters
            .upsert(field.into(), Map::from_entry("$lt", value));
    }

    /// Adds a filter with the condition for a field not greater than the value.
    #[inline]
    pub fn and_le<S, T>(&mut self, field: S, value: T)
    where
        S: Into<String>,
        T: Into<JsonValue>,
    {
        self.filters
            .upsert(field.into(), Map::from_entry("$le", value));
    }

    /// Adds a filter with the condition for a field greater than the value.
    #[inline]
    pub fn and_gt<S, T>(&mut self, field: S, value: T)
    where
        S: Into<String>,
        T: Into<JsonValue>,
    {
        self.filters
            .upsert(field.into(), Map::from_entry("$lt", value));
    }

    /// Adds a filter with the condition for a field not less than the value.
    #[inline]
    pub fn and_ge<S, T>(&mut self, field: S, value: T)
    where
        S: Into<String>,
        T: Into<JsonValue>,
    {
        self.filters
            .upsert(field.into(), Map::from_entry("$ge", value));
    }

    /// Adds a filter with the condition for a field whose value is in the list.
    #[inline]
    pub fn and_in<S, T>(&mut self, field: S, list: &[T])
    where
        S: Into<String>,
        T: Into<JsonValue> + Clone,
    {
        self.filters
            .upsert(field.into(), Map::from_entry("$in", list));
    }

    /// Adds a filter with the condition for a field whose value is not in the list.
    #[inline]
    pub fn and_not_in<S, T>(&mut self, field: S, list: &[T])
    where
        S: Into<String>,
        T: Into<JsonValue> + Clone,
    {
        self.filters
            .upsert(field.into(), Map::from_entry("$nin", list));
    }

    /// Adds a sort with the specific order.
    #[inline]
    pub fn order_by(&mut self, field: impl Into<SharedString>, descending: bool) -> &mut Self {
        self.sort_order.push((field.into(), descending));
        self
    }

    /// Adds a sort with the ascending order.
    #[inline]
    pub fn order_asc(&mut self, field: impl Into<SharedString>) -> &mut Self {
        self.sort_order.push((field.into(), false));
        self
    }

    /// Adds a sort with the descending order.
    #[inline]
    pub fn order_desc(&mut self, field: impl Into<SharedString>) -> &mut Self {
        self.sort_order.push((field.into(), true));
        self
    }

    /// Sets the offset.
    #[inline]
    pub fn offset(&mut self, offset: usize) -> &mut Self {
        self.offset = offset;
        self
    }

    /// Sets the limit.
    #[inline]
    pub fn limit(&mut self, limit: usize) -> &mut Self {
        self.limit = limit;
        self
    }

    /// Constructs an instance of `Query`.
    #[inline]
    pub fn build(self) -> Query {
        Query {
            fields: self.fields,
            filters: self.filters,
            sort_order: self.sort_order,
            offset: self.offset,
            limit: self.limit,
        }
    }
}
