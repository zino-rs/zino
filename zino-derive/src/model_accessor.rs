use super::parser;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::DeriveInput;

/// Parses the token stream for the `ModelAccessor` trait derivation.
pub(super) fn parse_token_stream(input: DeriveInput) -> TokenStream {
    // Model name
    let name = input.ident;

    // Parsing struct attributes
    let mut composite_constraints = Vec::new();
    for attr in input.attrs.iter() {
        for (key, value) in parser::parse_schema_attr(attr).into_iter() {
            if key == "unique_on" {
                if let Some(value) = value {
                    let mut fields = Vec::new();
                    let column_values = value
                        .trim_start_matches('(')
                        .trim_end_matches(')')
                        .split(',')
                        .map(|s| {
                            let field = s.trim();
                            let field_ident = format_ident!("{}", field);
                            fields.push(field);
                            quote! {
                                (#field, self.#field_ident.to_string().into())
                            }
                        })
                        .collect::<Vec<_>>();
                    let composite_field = fields.join("_");
                    composite_constraints.push(quote! {
                        let columns = vec![#(#column_values),*];
                        if !self.is_unique_on(columns).await? {
                            validation.record(#composite_field, "the composite values should be unique");
                        }
                    });
                }
            }
        }
    }

    // Parsing field attributes
    let mut column_methods = Vec::new();
    let mut snapshot_fields = Vec::new();
    let mut snapshot_entries = Vec::new();
    let mut field_constraints = Vec::new();
    let mut ignored_list_fields = Vec::new();
    let mut list_query_methods = Vec::new();
    let mut fetched_queries = Vec::new();
    let mut fetched_one_queries = Vec::new();
    let mut sample_queries = Vec::new();
    let mut soft_delete_updates = Vec::new();
    let mut lock_updates = Vec::new();
    let mut archive_updates = Vec::new();
    let mut primary_key_type = String::from("Uuid");
    let mut primary_key_name = String::from("id");
    let mut model_references: HashMap<String, Vec<String>> = HashMap::new();
    let mut populated_field_mappings: HashMap<String, String> = HashMap::new();
    for field in parser::parse_struct_fields(input.data) {
        let type_name = parser::get_type_name(&field.ty);
        if let Some(ident) = field.ident {
            let name = ident.to_string();
            let mut field_alias = None;
            for attr in field.attrs.iter() {
                let type_name = type_name.as_str();
                let arguments = parser::parse_schema_attr(attr);
                let is_readable = arguments.iter().all(|arg| arg.0 != "write_only");
                for (key, value) in arguments.into_iter() {
                    match key.as_str() {
                        "alias" => {
                            field_alias = value;
                        }
                        "primary_key" => {
                            primary_key_name.clone_from(&name);
                        }
                        "snapshot" => {
                            let field = name.clone();
                            let field_ident = format_ident!("{}", field);
                            if matches!(type_name, "Uuid" | "Decimal") {
                                snapshot_entries.push(quote! {
                                    snapshot.upsert(#field, self.#field_ident.to_string());
                                });
                            } else if type_name == "Option<Uuid>" {
                                snapshot_entries.push(quote! {
                                    let snapshot_value = self.#field_ident
                                        .map(|v| v.to_string());
                                    snapshot.upsert(#field, snapshot_value);
                                });
                            } else if type_name == "Vec<Uuid>" {
                                snapshot_entries.push(quote! {
                                    let snapshot_value = self.#field_ident.iter()
                                        .map(|v| v.to_string())
                                        .collect::<Vec<_>>();
                                    snapshot.upsert(#field, snapshot_value);
                                });
                            } else {
                                snapshot_entries.push(quote! {
                                    snapshot.upsert(#field, self.#field_ident.clone());
                                });
                            }
                            snapshot_fields.push(field);
                        }
                        "reference" => {
                            if let Some(value) = value {
                                let model_ident = format_ident!("{}", value);
                                if type_name == "Uuid" {
                                    field_constraints.push(quote! {
                                        let values = vec![self.#ident.to_string()];
                                        let data = <#model_ident>::filter(values).await?;
                                        if data.len() != 1 {
                                            validation.record(#name, "it is a nonexistent value");
                                        }
                                    });
                                } else if matches!(type_name, "Option<Uuid>" | "Option<String>") {
                                    field_constraints.push(quote! {
                                            if let Some(value) = self.#ident {
                                                let values = vec![value.to_string()];
                                                let data = <#model_ident>::filter(values).await?;
                                                if data.len() != 1 {
                                                    validation.record(#name, "it is a nonexistent value");
                                                }
                                            }
                                        });
                                } else if matches!(type_name, "Vec<Uuid>" | "Vec<String>") {
                                    field_constraints.push(quote! {
                                            let values = self.#ident
                                                .iter()
                                                .map(|v| v.to_string())
                                                .collect::<Vec<_>>();
                                            let length = values.len();
                                            if length > 0 {
                                                let data = <#model_ident>::filter(values).await?;
                                                if data.len() != length {
                                                    validation.record(#name, "there are nonexistent values");
                                                }
                                            }
                                        });
                                } else if parser::check_vec_type(type_name) {
                                    field_constraints.push(quote! {
                                            let values = self.#ident.clone();
                                            let length = values.len();
                                            if length > 0 {
                                                let data = <#model_ident>::filter(values).await?;
                                                if data.len() != length {
                                                    validation.record(#name, "there are nonexistent values");
                                                }
                                            }
                                        });
                                } else if parser::check_option_type(type_name) {
                                    field_constraints.push(quote! {
                                            if let Some(value) = self.#ident {
                                                let values = vec![value.clone()];
                                                let data = <#model_ident>::filter(values).await?;
                                                if data.len() != 1 {
                                                    validation.record(#name, "it is a nonexistent value");
                                                }
                                            }
                                        });
                                } else {
                                    field_constraints.push(quote! {
                                        let values = vec![self.#ident.clone()];
                                        let data = <#model_ident>::filter(values).await?;
                                        if data.len() != 1 {
                                            validation.record(#name, "it is a nonexistent value");
                                        }
                                    });
                                }
                                if let Some(vec) = model_references.get_mut(&value) {
                                    vec.push(name.clone());
                                } else {
                                    model_references.insert(value, vec![name.clone()]);
                                }
                                if parser::check_vec_type(type_name) {
                                    sample_queries.push(quote! {
                                        if let Some(col) = Self::get_column(#name) {
                                            let size = col.random_size();
                                            let values = <#model_ident>::sample(size).await?;
                                            associations.upsert(#name, values);
                                        }
                                    });
                                } else {
                                    sample_queries.push(quote! {
                                        if let Some(col) = Self::get_column(#name) {
                                            let size = col.random_size();
                                            let values = <#model_ident>::sample(size).await?;
                                            associations.upsert(#name, values.first().cloned());
                                        }
                                    });
                                }
                            }
                        }
                        "fetch_as" => {
                            if let Some(value) = value {
                                let populated_field = [&name, "_populated"].concat();
                                populated_field_mappings.insert(populated_field, value);
                            }
                        }
                        "unique" => {
                            if type_name == "Uuid" {
                                field_constraints.push(quote! {
                                        let value = self.#ident;
                                        if !value.is_nil() {
                                            let columns = vec![(#name, value.to_string().into())];
                                            if !self.is_unique_on(columns).await? {
                                                let message = format!("the value `{value}` is not unique");
                                                validation.record(#name, message);
                                            }
                                        }
                                    });
                            } else if type_name == "String" {
                                field_constraints.push(quote! {
                                        let value = self.#ident.as_str();
                                        if !value.is_empty() {
                                            let columns = vec![(#name, value.into())];
                                            if !self.is_unique_on(columns).await? {
                                                let message = format!("the value `{value}` is not unique");
                                                validation.record(#name, message);
                                            }
                                        }
                                    });
                            } else if type_name == "Option<String>" {
                                field_constraints.push(quote! {
                                        if let Some(value) = self.#ident.as_deref() && !value.is_empty() {
                                            let columns = vec![(#name, value.into())];
                                            if !self.is_unique_on(columns).await? {
                                                let message = format!("the value `{value}` is not unique");
                                                validation.record(#name, message);
                                            }
                                        }
                                    });
                            } else if type_name == "Option<Uuid>" {
                                field_constraints.push(quote! {
                                        if let Some(value) = self.#ident && !value.is_nil() {
                                            let columns = vec![(#name, value.to_string().into())];
                                            if !self.is_unique_on(columns).await? {
                                                let message = format!("the value `{value}` is not unique");
                                                validation.record(#name, message);
                                            }
                                        }
                                    });
                            } else if parser::check_option_type(type_name) {
                                field_constraints.push(quote! {
                                        if let Some(value) = self.#ident {
                                            let columns = vec![(#name, value.into())];
                                            if !self.is_unique_on(columns).await? {
                                                let message = format!("the value `{value}` is not unique");
                                                validation.record(#name, message);
                                            }
                                        }
                                    });
                            } else {
                                field_constraints.push(quote! {
                                    let value = self.#ident;
                                    let columns = vec![(#name, value.into())];
                                    if !self.is_unique_on(columns).await? {
                                        let message = format!("the value `{value}` is not unique");
                                        validation.record(#name, message);
                                    }
                                });
                            }
                        }
                        "not_null" if is_readable => {
                            if type_name == "String" {
                                field_constraints.push(quote! {
                                    if self.#ident.is_empty() {
                                        validation.record(#name, "it should be nonempty");
                                    }
                                });
                            } else if type_name == "Uuid" {
                                field_constraints.push(quote! {
                                    if self.#ident.is_nil() {
                                        validation.record(#name, "it should not be nil");
                                    }
                                });
                            }
                        }
                        "nonempty" if is_readable => {
                            if parser::check_vec_type(type_name)
                                || matches!(type_name, "String" | "Map")
                            {
                                field_constraints.push(quote! {
                                    if self.#ident.is_empty() {
                                        validation.record(#name, "it should be nonempty");
                                    }
                                });
                            }
                        }
                        "validator" if is_readable && type_name == "String" => {
                            if let Some(value) = value {
                                if let Some((validator, validator_fn)) = value.split_once("::") {
                                    let validator_ident = format_ident!("{}", validator);
                                    let validator_fn_ident = format_ident!("{}", validator_fn);
                                    field_constraints.push(quote! {
                                            if !self.#ident.is_empty() {
                                                let validator = <#validator_ident>::#validator_fn_ident();
                                                if let Err(err) = validator.validate(self.#ident.as_str()) {
                                                    validation.record_fail(#name, err);
                                                }
                                            }
                                        });
                                } else {
                                    let validator_ident = format_ident!("{}", value);
                                    field_constraints.push(quote! {
                                            if !self.#ident.is_empty() {
                                                if let Err(err) = #validator_ident.validate(self.#ident.as_str()) {
                                                    validation.record_fail(#name, err);
                                                }
                                            }
                                        });
                                }
                            }
                        }
                        "format" if is_readable && type_name == "String" => {
                            if let Some(value) = value {
                                field_constraints.push(quote! {
                                        if !self.#ident.is_empty() {
                                            validation.validate_format(#name, self.#ident.as_str(), #value);
                                        }
                                    });
                            }
                        }
                        "enum_values" => {
                            if let Some(value) = value {
                                let values = value.split('|').map(|s| s.trim()).collect::<Vec<_>>();
                                if type_name == "String" {
                                    field_constraints.push(quote! {
                                            if !self.#ident.is_empty() {
                                                let values = [#(#values),*];
                                                let value = self.#ident.as_str();
                                                if !values.contains(&value) {
                                                    let message = format!("the value `{value}` is not allowed");
                                                    validation.record(#name, message);
                                                }
                                            }
                                        });
                                } else if type_name == "Vec<String>" {
                                    field_constraints.push(quote! {
                                            let values = [#(#values),*];
                                            for value in self.#ident.iter() {
                                                if !values.contains(&value.as_str()) {
                                                    let message = format!("the value `{value}` is not allowed");
                                                    validation.record(#name, message);
                                                    break;
                                                }
                                            }
                                        });
                                }
                            }
                        }
                        "length" if is_readable => {
                            let length = value
                                .and_then(|s| s.parse::<usize>().ok())
                                .unwrap_or_default();
                            if type_name == "String" {
                                field_constraints.push(quote! {
                                    let length = #length;
                                    if self.#ident.len() != length {
                                        let message = format!("the length should be {length}");
                                        validation.record(#name, message);
                                    }
                                });
                            } else if type_name == "Option<String>" {
                                field_constraints.push(quote! {
                                    let length = #length;
                                    if let Some(ref s) = self.#ident && s.len() != length {
                                        let message = format!("the length should be {length}");
                                        validation.record(#name, message);
                                    }
                                });
                            }
                        }
                        "max_length" if is_readable => {
                            let length = value
                                .and_then(|s| s.parse::<usize>().ok())
                                .unwrap_or_default();
                            if type_name == "String" {
                                field_constraints.push(quote! {
                                        let length = #length;
                                        if self.#ident.len() > length {
                                            let message = format!("the length should be at most {length}");
                                            validation.record(#name, message);
                                        }
                                    });
                            } else if type_name == "Option<String>" {
                                field_constraints.push(quote! {
                                        let length = #length;
                                        if let Some(ref s) = self.#ident && s.len() > length {
                                            let message = format!("the length should be at most {length}");
                                            validation.record(#name, message);
                                        }
                                    });
                            }
                        }
                        "min_length" if is_readable => {
                            let length = value
                                .and_then(|s| s.parse::<usize>().ok())
                                .unwrap_or_default();
                            if type_name == "String" {
                                field_constraints.push(quote! {
                                        let length = #length;
                                        if self.#ident.len() < length {
                                            let message = format!("the length should be at least {length}");
                                            validation.record(#name, message);
                                        }
                                    });
                            } else if type_name == "Option<String>" {
                                field_constraints.push(quote! {
                                        let length = #length;
                                        if let Some(ref s) = self.#ident && s.len() < length {
                                            let message = format!("the length should be at least {length}");
                                            validation.record(#name, message);
                                        }
                                    });
                            }
                        }
                        "max_items" => {
                            if let Some(length) = value.and_then(|s| s.parse::<usize>().ok()) {
                                if parser::check_vec_type(type_name) {
                                    field_constraints.push(quote! {
                                        let length = #length;
                                        if self.#ident.len() > length {
                                            let message = format!("the length should be at most {length}");
                                            validation.record(#name, message);
                                        }
                                    });
                                }
                            }
                        }
                        "min_items" => {
                            if let Some(length) = value.and_then(|s| s.parse::<usize>().ok()) {
                                if parser::check_vec_type(type_name) {
                                    field_constraints.push(quote! {
                                        let length = #length;
                                        if self.#ident.len() < length {
                                            let message = format!("the length should be at least {length}");
                                            validation.record(#name, message);
                                        }
                                    });
                                }
                            }
                        }
                        "unique_items" => {
                            if parser::check_vec_type(type_name) {
                                field_constraints.push(quote! {
                                    let slice = self.#ident.as_slice();
                                    for index in 1..slice.len() {
                                        if slice[index..].contains(&slice[index - 1]) {
                                            let message = format!("array items should be unique");
                                            validation.record(#name, message);
                                            break;
                                        }
                                    }
                                });
                            }
                        }
                        "less_than" => {
                            if let Some(value) = value {
                                if let Some((field_type, field_type_fn)) = value.split_once("::") {
                                    let field_type_ident = format_ident!("{}", field_type);
                                    let field_type_fn_ident = format_ident!("{}", field_type_fn);
                                    field_constraints.push(quote! {
                                            let field_value = <#field_type_ident>::#field_type_fn_ident();
                                            if self.#ident >= field_value {
                                                let message = format!("should be less than `{field_value}`");
                                                validation.record(#name, message);
                                            }
                                        });
                                } else {
                                    let field_ident = format_ident!("{}", value);
                                    field_constraints.push(quote! {
                                            let field_value = self.#field_ident;
                                            if self.#ident >= field_value {
                                                let message = format!("should be less than `{field_value}`");
                                                validation.record(#name, message);
                                            }
                                        });
                                }
                            }
                        }
                        "greater_than" => {
                            if let Some(value) = value {
                                if let Some((field_type, field_type_fn)) = value.split_once("::") {
                                    let field_type_ident = format_ident!("{}", field_type);
                                    let field_type_fn_ident = format_ident!("{}", field_type_fn);
                                    field_constraints.push(quote! {
                                            let field_value = <#field_type_ident>::#field_type_fn_ident();
                                            if self.#ident <= field_value {
                                                let message = format!("should be greater than `{field_value}`");
                                                validation.record(#name, message);
                                            }
                                        });
                                } else {
                                    let field_ident = format_ident!("{}", value);
                                    field_constraints.push(quote! {
                                            let field_value = self.#field_ident;
                                            if self.#ident <= field_value {
                                                let message = format!("should be greater than `{field_value}`");
                                                validation.record(#name, message);
                                            }
                                        });
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }
            if primary_key_name == name {
                primary_key_type = type_name;
            } else {
                let field_name = name.as_str();
                let field_ident = format_ident!("{}", field_name);
                let mut snapshot_field = None;
                match field_alias.as_deref().unwrap_or(field_name) {
                    "name" if type_name == "String" => {
                        let method = quote! {
                            #[inline]
                            fn #field_ident(&self) -> &str {
                                self.#field_ident.as_ref()
                            }
                        };
                        column_methods.push(method);
                        snapshot_field = Some(field_name);
                    }
                    "namespace" | "visibility" | "description" if type_name == "String" => {
                        let method = quote! {
                            #[inline]
                            fn #field_ident(&self) -> &str {
                                self.#field_ident.as_ref()
                            }
                        };
                        column_methods.push(method);
                    }
                    "status" if type_name == "String" => {
                        let method = quote! {
                            #[inline]
                            fn #field_ident(&self) -> &str {
                                self.#field_ident.as_ref()
                            }
                        };
                        column_methods.push(method);
                        snapshot_field = Some(field_name);
                        list_query_methods.push(quote! {
                            query.add_filter(#field_name, Map::from_entry("$ne", "Deleted"));
                        });
                        soft_delete_updates.push(quote! {
                            updates.upsert(#field_name, "Deleted");
                        });
                        lock_updates.push(quote! {
                            updates.upsert(#field_name, "Locked");
                        });
                        archive_updates.push(quote! {
                            updates.upsert(#field_name, "Archived");
                        });
                    }
                    "extra" if type_name == "Map" => {
                        let method = quote! {
                            #[inline]
                            fn #field_ident(&self) -> Option<&Map> {
                                let map = &self.#field_ident;
                                (!map.is_empty()).then_some(map)
                            }
                        };
                        column_methods.push(method);
                        ignored_list_fields.push(field_name.to_owned());
                    }
                    "created_at" if type_name == "DateTime" => {
                        let method = quote! {
                            #[inline]
                            fn #field_ident(&self) -> DateTime {
                                self.#field_ident
                            }
                        };
                        column_methods.push(method);
                    }
                    "updated_at" if type_name == "DateTime" => {
                        let method = quote! {
                            #[inline]
                            fn #field_ident(&self) -> DateTime {
                                self.#field_ident
                            }
                        };
                        column_methods.push(method);
                        snapshot_field = Some(field_name);
                        list_query_methods.push(quote! {
                            query.order_desc(#field_name);
                        });
                    }
                    "deleted_at" if type_name == "Option<DateTime>" => {
                        let method = quote! {
                            #[inline]
                            fn #field_ident(&self) -> Option<DateTime> {
                                self.#field_ident
                            }
                        };
                        column_methods.push(method);
                        column_methods.push(quote! {
                            #[inline]
                            fn is_deleted(&self) -> bool {
                                self.#field_ident.is_some()
                            }
                        });
                        list_query_methods.push(quote! {
                            query.add_filter(#field_name, "null");
                        });
                        soft_delete_updates.push(quote! {
                            updates.upsert(#field_name, DateTime::now().format_timestamp());
                        });
                    }
                    "version" if type_name == "u64" => {
                        let method = quote! {
                            #[inline]
                            fn #field_ident(&self) -> u64 {
                                self.#field_ident
                            }
                        };
                        column_methods.push(method);
                        snapshot_field = Some(field_name);
                    }
                    "edition" if type_name == "u32" => {
                        let method = quote! {
                            #[inline]
                            fn #field_ident(&self) -> u32 {
                                self.#field_ident
                            }
                        };
                        column_methods.push(method);
                    }
                    "is_deleted" if type_name == "bool" => {
                        column_methods.push(quote! {
                            #[inline]
                            fn is_deleted(&self) -> bool {
                                self.#field_ident
                            }
                        });
                        list_query_methods.push(quote! {
                            query.add_filter(#field_name, false);
                        });
                        soft_delete_updates.push(quote! {
                            updates.upsert(#field_name, true);
                        });
                    }
                    "is_locked" if type_name == "bool" => {
                        column_methods.push(quote! {
                            #[inline]
                            fn is_locked(&self) -> bool {
                                self.#field_ident
                            }
                        });
                        lock_updates.push(quote! {
                            updates.upsert(#field_name, true);
                        });
                    }
                    "is_archived" if type_name == "bool" => {
                        column_methods.push(quote! {
                            #[inline]
                            fn is_archived(&self) -> bool {
                                self.#field_ident
                            }
                        });
                        archive_updates.push(quote! {
                            updates.upsert(#field_name, true);
                        });
                    }
                    _ => (),
                }
                if let Some(field_name) = snapshot_field {
                    let field_ident = format_ident!("{}", field_name);
                    snapshot_entries.push(quote! {
                        snapshot.upsert(#field_name, self.#field_ident.clone());
                    });
                    snapshot_fields.push(field_name.to_owned());
                }
                if ignored_list_fields.is_empty() {
                    list_query_methods.push(quote! {
                        query.deny_fields(Self::write_only_fields());
                    });
                } else {
                    list_query_methods.push(quote! {
                        query.deny_fields(&[
                            Self::write_only_fields(),
                            &[#(#ignored_list_fields),*],
                        ].concat());
                    });
                }
            }
        }
    }
    fetched_queries.push(quote! {
        let mut models = Self::find::<Map>(query).await?;
        for model in models.iter_mut() {
            Self::after_decode(model).await?;
            translate_enabled.then(|| Self::translate_model(model));
        }
    });
    fetched_one_queries.push(quote! {
        let mut model = Self::find_by_id::<Map>(id)
            .await?
            .ok_or_else(|| zino_core::warn!("404 Not Found: cannot find the model `{}`", id))?;
        Self::after_decode(&mut model).await?;
        Self::translate_model(&mut model);
    });
    if !model_references.is_empty() {
        for (model, ref_fields) in model_references.into_iter() {
            let model_ident = format_ident!("{}", model);
            let populated_query = quote! {
                let mut query = #model_ident::default_snapshot_query();
                query.set_extra_flag("translate", translate_enabled);
                #model_ident::populate(&mut query, &mut models, &[#(#ref_fields),*]).await?;
            };
            let populated_one_query = quote! {
                let mut query = #model_ident::default_query();
                query.set_extra_flag("translate", true);
                #model_ident::populate_one(&mut query, &mut model, &[#(#ref_fields),*]).await?;
            };
            fetched_queries.push(populated_query);
            fetched_one_queries.push(populated_one_query);
        }
    }
    if !populated_field_mappings.is_empty() {
        for (key, new_key) in populated_field_mappings {
            let populated_query = quote! {
                for model in &mut models {
                    if let Some(value) = model.remove(#key) {
                        model.upsert(#new_key, value);
                    }
                }
            };
            let populated_one_query = quote! {
                if let Some(value) = model.remove(#key) {
                    model.upsert(#new_key, value);
                }
            };
            fetched_queries.push(populated_query);
            fetched_one_queries.push(populated_one_query);
        }
    }
    fetched_queries.push(quote! { Ok(models) });
    fetched_one_queries.push(quote! { Ok(model) });

    // Output
    let model_primary_key_type = format_ident!("{}", primary_key_type);
    let model_primary_key = format_ident!("{}", primary_key_name);
    quote! {
        use zino_core::{
            model::{Mutation, Query},
            orm::ModelHelper as _,
            validation::Validation as ZinoValidation,
            Map as ZinoMap,
        };

        impl zino_core::orm::ModelAccessor<#model_primary_key_type> for #name {
            #[inline]
            fn id(&self) -> &#model_primary_key_type {
                &self.#model_primary_key
            }

            #(#column_methods)*

            fn snapshot(&self) -> ZinoMap {
                let mut snapshot = ZinoMap::new();
                snapshot.upsert(Self::PRIMARY_KEY_NAME, self.primary_key_value());
                #(#snapshot_entries)*
                snapshot
            }

            fn soft_delete_mutation(&self) -> Mutation {
                let mut mutation = Self::default_mutation();
                let mut updates = self.next_edition_updates();
                #(#soft_delete_updates)*
                mutation.append_updates(&mut updates);
                mutation
            }

            fn lock_mutation(&self) -> Mutation {
                let mut mutation = Self::default_mutation();
                let mut updates = self.next_edition_updates();
                #(#lock_updates)*
                mutation.append_updates(&mut updates);
                mutation
            }

            fn archive_mutation(&self) -> Mutation {
                let mut mutation = Self::default_mutation();
                let mut updates = self.next_edition_updates();
                #(#archive_updates)*
                mutation.append_updates(&mut updates);
                mutation
            }

            fn default_snapshot_query() -> Query {
                let mut query = Query::default();
                let fields = [
                    Self::PRIMARY_KEY_NAME,
                    #(#snapshot_fields),*
                ];
                query.allow_fields(&fields);
                query.deny_fields(Self::write_only_fields());
                query
            }

            fn default_list_query() -> Query {
                let mut query = Query::default();
                query.allow_fields(Self::fields());
                #(#list_query_methods)*
                query
            }

            async fn check_constraints(&self) -> Result<ZinoValidation, ZinoError> {
                let mut validation = ZinoValidation::new();
                if self.id() == &<#model_primary_key_type>::default()
                    && !Self::primary_key_column().auto_increment()
                {
                    validation.record(Self::PRIMARY_KEY_NAME, "should not be a default value");
                }
                #(#composite_constraints)*
                #(#field_constraints)*
                Ok(validation)
            }

            async fn fetch(query: &Query) -> Result<Vec<ZinoMap>, ZinoError> {
                let translate_enabled = query.translate_enabled();
                #(#fetched_queries)*
            }

            async fn fetch_by_id(id: &#model_primary_key_type) -> Result<ZinoMap, ZinoError> {
                #(#fetched_one_queries)*
            }

            async fn random_associations() -> Result<ZinoMap, ZinoError> {
                let mut associations = ZinoMap::new();
                #(#sample_queries)*
                Ok(associations)
            }
        }
    }
}
