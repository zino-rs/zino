use super::parser;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{Data, DeriveInput, Fields};

/// Parses the token stream for the `ModelHooks` trait derivation.
pub(super) fn parse_token_stream(input: DeriveInput) -> TokenStream {
    // Model name
    let name = input.ident;

    // Parsing field attributes
    let mut embedding_queries = Vec::new();
    if let Data::Struct(data) = input.data
        && let Fields::Named(fields) = data.fields
    {
        let mut model_mappings: HashMap<String, String> = HashMap::new();
        let mut field_embeddings: HashMap<String, Vec<(String, String)>> = HashMap::new();
        for field in fields.named.into_iter() {
            let type_name = parser::get_type_name(&field.ty);
            if let Some(ident) = field.ident
                && !type_name.is_empty()
            {
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
                            "referenced_field" => {
                                if let Some(value) = value {
                                    field_mapping = Some((name.clone(), value));
                                }
                            }
                            _ => (),
                        }
                    }
                }
                if let Some(correlatation_field) = correlatation_field
                    && let Some(field_mapping) = field_mapping
                {
                    if let Some(vec) = field_embeddings.get_mut(&correlatation_field) {
                        vec.push(field_mapping);
                    } else {
                        field_embeddings.insert(correlatation_field, vec![field_mapping]);
                    }
                }
            }
        }
        for (correlatation_field, model) in model_mappings.into_iter() {
            if let Some(field_mappings) = field_embeddings.get(&correlatation_field) {
                let model_ident = format_ident!("{}", model);
                for (field, referenced_field) in field_mappings {
                    embedding_queries.push(quote! {
                        if data.contains_key(#correlatation_field) {
                            let id = &self.#correlatation_field;
                            let value = <#model_ident>::find_scalar_by_id(&id, #referenced_field).await?;
                            self.#field = value;
                            data.upsert(#field, value.to_string());
                        }
                    });
                }
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
