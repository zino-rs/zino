use super::QueryOrder;
use crate::{
    extension::{JsonObjectExt, JsonValueExt},
    validation::Validation,
    JsonValue, Map, SharedString,
};

/// A query type for models.
#[derive(Debug, Clone)]
pub struct Query {
    /// Projection fields.
    fields: Vec<String>,
    /// Filters.
    filters: Map,
    /// Sort order.
    sort_order: Vec<QueryOrder>,
    /// Offset.
    offset: usize,
    /// Limit.
    limit: usize,
    /// Extra flags.
    extra: Map,
}

impl Query {
    /// Creates a new instance.
    #[inline]
    pub fn new(filters: impl Into<JsonValue>) -> Self {
        let filters = filters.into().into_map_opt().unwrap_or_default();
        Self {
            fields: Vec::new(),
            filters,
            sort_order: Vec::new(),
            offset: 0,
            limit: 0,
            extra: Map::new(),
        }
    }

    /// Creates a new instance with the entry.
    #[inline]
    pub fn from_entry(key: impl Into<String>, value: impl Into<JsonValue>) -> Self {
        Self::new(Map::from_entry(key, value))
    }

    /// Updates the query using the json object and returns the validation result.
    #[must_use]
    pub fn read_map(&mut self, data: &Map) -> Validation {
        let mut validation = Validation::new();
        let mut pagination_current_page = None;
        let filters = &mut self.filters;
        let extra = &mut self.extra;
        for (key, value) in data.iter().filter(|(_, v)| !v.is_ignorable()) {
            match key.as_str() {
                "fields" | "columns" => {
                    if let Some(fields) = value.parse_str_array() {
                        self.fields.clear();
                        self.fields.extend(fields.into_iter().map(|s| s.to_owned()));
                    }
                }
                "order_by" | "sort_by" => {
                    if let Some(sort_order) = value.parse_str_array() {
                        self.sort_order.clear();
                        self.sort_order.extend(sort_order.into_iter().map(|s| {
                            if let Some(sort) = s.strip_suffix("|asc") {
                                QueryOrder::new(sort.to_owned(), false)
                            } else if let Some(sort) = s.strip_suffix("|desc") {
                                QueryOrder::new(sort.to_owned(), true)
                            } else {
                                QueryOrder::new(s.to_owned(), true)
                            }
                        }));
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
                    // Parses as `isize` so that it can accept `-1`
                    if let Some(result) = value.parse_isize() {
                        match result {
                            Ok(limit) => self.limit = usize::MIN.saturating_add_signed(limit),
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
                "populate" | "translate" | "show_deleted" | "validate_only" | "no_check" => {
                    if let Some(result) = value.parse_bool() {
                        match result {
                            Ok(flag) => {
                                extra.upsert(key, flag);
                            }
                            Err(err) => validation.record_fail(key.to_owned(), err),
                        }
                    }
                }
                "timestamp" | "nonce" | "signature" => {
                    extra.upsert(key, value.clone());
                }
                _ => {
                    if let Some(value) = value.as_str().filter(|&s| s != "all") {
                        if key.starts_with('$') {
                            if let Some(expr) = value.strip_prefix('(') {
                                filters.upsert(key, Self::parse_logical_query(expr));
                            } else {
                                filters.upsert(key, value);
                            }
                        } else if value.starts_with('$') {
                            if let Some((operator, value)) = value.split_once('.') {
                                filters.upsert(key, Map::from_entry(operator, value));
                            } else {
                                filters.upsert(key, value);
                            }
                        } else {
                            filters.upsert(key, value);
                        }
                    } else {
                        filters.upsert(key, value.clone());
                    }
                }
            }
        }
        if let Some(current_page) = pagination_current_page {
            self.offset = self.limit * current_page.saturating_sub(1);
        }
        validation
    }

    /// Parses the query expression with logical operators.
    fn parse_logical_query(expr: &str) -> Vec<Map> {
        let mut filters = Vec::new();
        for expr in expr.trim_end_matches(')').split(',') {
            if let Some((key, expr)) = expr.split_once('.') {
                if let Some((operator, value)) = expr.split_once('.') {
                    let value = if value.starts_with('$') {
                        if let Some((operator, expr)) = value.split_once('(') {
                            Map::from_entry(operator, Self::parse_logical_query(expr)).into()
                        } else {
                            JsonValue::from(value)
                        }
                    } else {
                        JsonValue::from(value)
                    };
                    let filter = Map::from_entry(key, Map::from_entry(operator, value));
                    filters.push(filter);
                }
            }
        }
        filters
    }

    /// Sets the fields.
    #[inline]
    pub fn set_fields(&mut self, fields: Vec<String>) {
        self.fields = fields;
    }

    /// Retains the projection fields in the allow list.
    /// If the projection fields are empty, it will be set to the list.
    #[inline]
    pub fn allow_fields(&mut self, fields: &[&str]) {
        if self.fields.is_empty() {
            self.fields.extend(fields.iter().map(|&key| key.to_owned()));
        } else {
            self.fields.retain(|field| fields.contains(&field.as_str()))
        }
    }

    /// Removes the projection fields in the deny list.
    #[inline]
    pub fn deny_fields(&mut self, fields: &[&str]) {
        self.fields
            .retain(|field| !fields.contains(&field.as_str()))
    }

    /// Adds a projection field with the alias.
    #[inline]
    pub fn add_field_alias(&mut self, expr: impl Into<String>, alias: impl Into<String>) {
        self.fields.push([alias.into(), expr.into()].join(":"));
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

    /// Removes a query filter with the key.
    #[inline]
    pub fn remove_filter(&mut self, key: &str) -> Option<JsonValue> {
        self.filters.remove(key)
    }

    /// Sets the extra flag.
    #[inline]
    pub fn set_extra_flag(&mut self, key: impl Into<String>, value: impl Into<JsonValue>) {
        self.extra.upsert(key, value);
    }

    /// Appends the extra flags.
    #[inline]
    pub fn append_extra_flags(&mut self, flags: &mut Map) {
        self.extra.append(flags);
    }

    /// Sets the sort order.
    #[inline]
    pub fn set_order(&mut self, sort_order: Vec<QueryOrder>) {
        self.sort_order = sort_order;
    }

    /// Adds a query order.
    #[inline]
    pub fn order_by(&mut self, field: impl Into<SharedString>, descending: bool) {
        let field = field.into();
        self.sort_order
            .retain(|order| order.field() != field.as_ref());
        self.sort_order.push(QueryOrder::new(field, descending));
    }

    /// Adds a query order with an extra flag to indicate whether the nulls appear first or last.
    pub fn order_by_with_nulls(
        &mut self,
        field: impl Into<SharedString>,
        descending: bool,
        nulls_first: bool,
    ) {
        let field = field.into();
        self.sort_order
            .retain(|order| order.field() != field.as_ref());

        let mut order = QueryOrder::new(field, descending);
        if nulls_first {
            order.set_nulls_first();
        } else {
            order.set_nulls_last();
        }
        self.sort_order.push(order);
    }

    /// Adds a query order with an ascending order.
    #[inline]
    pub fn order_asc(&mut self, field: impl Into<SharedString>) {
        let field = field.into();
        self.sort_order
            .retain(|order| order.field() != field.as_ref());
        self.sort_order.push(QueryOrder::new(field, false));
    }

    /// Adds a query order with an descending order.
    #[inline]
    pub fn order_desc(&mut self, field: impl Into<SharedString>) {
        let field = field.into();
        self.sort_order
            .retain(|order| order.field() != field.as_ref());
        self.sort_order.push(QueryOrder::new(field, true));
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

    /// Disables the query limit.
    #[inline]
    pub fn disable_limit(&mut self) {
        self.limit = 0;
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

    /// Returns a reference to the sort order.
    #[inline]
    pub fn sort_order(&self) -> &[QueryOrder] {
        self.sort_order.as_slice()
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
        self.extra.get_bool(flag).is_some_and(|b| b)
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

    /// Returns `true` if the `show_deleted` flag has been enabled.
    #[inline]
    pub fn show_deleted(&self) -> bool {
        self.enabled("show_deleted")
    }

    /// Returns `true` if the `validate_only` flag has been enabled.
    #[inline]
    pub fn validate_only(&self) -> bool {
        self.enabled("validate_only")
    }

    /// Returns `true` if the `no_check` flag has been enabled.
    #[inline]
    pub fn no_check(&self) -> bool {
        self.enabled("no_check")
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
            extra: Map::new(),
        }
    }
}
