use syn::{Attribute, GenericArgument, Lit, Meta, NestedMeta, PathArguments, Type};

/// Returns the Postgres type name as a str.
pub(crate) fn get_type_name(ty: &Type) -> String {
    if let Type::Path(ty) = ty {
        let segment = ty.path.segments.last().unwrap();
        let type_name = segment.ident.to_string();
        if let PathArguments::AngleBracketed(ref generics) = segment.arguments {
            if let Some(GenericArgument::Type(ref ty)) = generics.args.first() {
                return type_name + "<" + &get_type_name(ty) + ">";
            }
        }
        type_name
    } else {
        String::new()
    }
}

/// Parses an attribute and returns a list of arguments
pub(crate) fn parse_attr(attr: &Attribute) -> Vec<(String, Option<String>)> {
    if let Ok(meta) = attr.parse_meta() {
        if let Some(ident) = meta.path().get_ident() && *ident != "schema" {
            return Vec::new();
        }
        if let Meta::List(list) = meta {
            let mut arguments = Vec::new();
            for nested_meta in list.nested.iter() {
                if let NestedMeta::Meta(meta) = nested_meta {
                    if let Some(ident) = meta.path().get_ident() {
                        let key = ident.to_string();
                        let value = match meta {
                            Meta::NameValue(value) => match value.lit {
                                Lit::Str(ref lit_str) => Some(lit_str.value()),
                                Lit::Bool(ref lit_bool) => Some(lit_bool.value.to_string()),
                                Lit::Int(ref lit_int) => Some(lit_int.base10_digits().to_string()),
                                _ => None,
                            },
                            _ => None,
                        };
                        arguments.push((key, value));
                    }
                }
            }
            return arguments;
        }
    }
    Vec::new()
}
