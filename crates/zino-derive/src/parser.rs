use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    punctuated::Punctuated, Attribute, Data, Expr, Field, Fields, GenericArgument, Lit, Meta,
    PathArguments, Token, Type,
};

/// Quotes the `Option<String>` value.
pub(super) fn quote_option_string(value: Option<String>) -> TokenStream {
    match value {
        Some(v) => quote! { Some(#v) },
        None => quote! { None },
    }
}

/// Parses the `Option<T>` type.
pub(super) fn parse_option_type(type_name: &str) -> Option<&str> {
    type_name
        .split_once('<')
        .filter(|&(t, s)| t == "Option" && s.ends_with('>'))
        .map(|(_, s)| s.trim_end_matches('>'))
}

/// Returns `true` if the type is `Vec<T>`.
pub(super) fn check_vec_type(type_name: &str) -> bool {
    type_name
        .split_once('<')
        .is_some_and(|(t, s)| t == "Vec" && s.ends_with('>'))
}

/// Returns `true` if the type is `Option<T>`.
pub(super) fn check_option_type(type_name: &str) -> bool {
    type_name
        .split_once('<')
        .is_some_and(|(t, s)| t == "Option" && s.ends_with('>'))
}

/// Returns the type name as a str.
pub(super) fn get_type_name(ty: &Type) -> String {
    if let Type::Path(ty) = ty {
        if let Some(segment) = ty.path.segments.last() {
            let type_name = segment.ident.to_string();
            if let PathArguments::AngleBracketed(ref generics) = segment.arguments {
                if let Some(GenericArgument::Type(ref ty)) = generics.args.first() {
                    return type_name + "<" + &get_type_name(ty) + ">";
                }
            }
            return type_name;
        }
    }
    String::new()
}

/// Parses an attribute and returns a list of arguments.
pub(super) fn parse_schema_attr(attr: &Attribute) -> Vec<(String, Option<String>)> {
    let mut arguments = Vec::new();
    if attr.path().is_ident("schema") {
        if let Ok(nested) = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) {
            for meta in nested {
                if let Some(ident) = meta.path().get_ident() {
                    let key = ident.to_string();
                    let value = if let Meta::NameValue(name_value) = meta {
                        if let Expr::Lit(expr_lit) = name_value.value {
                            match expr_lit.lit {
                                Lit::Str(ref lit_str) => Some(lit_str.value()),
                                Lit::Bool(ref lit_bool) => Some(lit_bool.value.to_string()),
                                Lit::Int(ref lit_int) => Some(lit_int.base10_digits().to_owned()),
                                _ => None,
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    arguments.push((key, value));
                }
            }
        }
    }
    arguments
}

/// Parses the struct data and returns a list of fields.
pub(super) fn parse_struct_fields(data: Data) -> Vec<Field> {
    if let Data::Struct(data) = data {
        if let Fields::Named(fields) = data.fields {
            return fields.named.into_iter().collect();
        }
    }
    Vec::new()
}
