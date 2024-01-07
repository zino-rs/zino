use super::parser;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::DeriveInput;

/// Parses the token stream for the `ModelHooks` trait derivation.
pub(super) fn parse_token_stream(input: DeriveInput) -> TokenStream {
    // Model name
    let name = input.ident;

    // Parsing field attributes
    let mut embedding_queries = Vec::new();
    let mut model_mappings: HashMap<String, String> = HashMap::new();
    let mut field_embeddings: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for field in parser::parse_struct_fields(input.data) {
        if let Some(ident) = field.ident {
            let name = ident.to_string();
            let mut correlatation_field = None;
            let mut field_mapping = None;
            for attr in field.attrs.iter() {
                let arguments = parser::parse_schema_attr(attr);
                for (key, value) in arguments.into_iter() {
                    match key.as_str() {
                        "reference" => {
                            if let Some(value) = value {
                                model_mappings.insert(name.clone(), value);
                            }
                        }
                        "correlates_with" => {
                            correlatation_field = value;
                        }
                        "foreign_field" => {
                            if let Some(value) = value {
                                field_mapping = Some((name.clone(), value));
                            }
                        }
                        _ => (),
                    }
                }
            }
            if let Some(correlatation_field) = correlatation_field {
                if let Some(field_mapping) = field_mapping {
                    if let Some(vec) = field_embeddings.get_mut(&correlatation_field) {
                        vec.push(field_mapping);
                    } else {
                        field_embeddings.insert(correlatation_field, vec![field_mapping]);
                    }
                }
            }
        }
    }
    for (correlatation_field, model) in model_mappings.into_iter() {
        if let Some(field_mappings) = field_embeddings.get(&correlatation_field) {
            let model_ident = format_ident!("{}", model);
            for (field, referenced_field) in field_mappings {
                let query = quote! {
                    if data.contains_key(#correlatation_field) {
                        let id = &self.#correlatation_field;
                        let value = <#model_ident>::find_scalar_by_id(&id, #referenced_field).await?;
                        self.#field = value;
                        data.upsert(#field, value.to_string());
                    }
                };
                embedding_queries.push(query);
            }
        }
    }
    if embedding_queries.is_empty() {
        quote! {
            use zino_core::model::ModelHooks;

            impl ModelHooks for #name {
                type Data = ();
                type Extension = ();
            }
        }
    } else {
        quote! {
            use zino_core::{model::{ModelHooks, QueryContext}, schema::ScalarQuery};

            impl ModelHooks for #name {
                type Data = ();
                type Extension = ();

                async fn after_validation(&mut self, data: &mut Map) -> Result<(), Error> {
                    #(#embedding_queries)*
                }
            }
        }
    }
}
