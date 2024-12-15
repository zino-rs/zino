use mime_guess::{
    mime::{APPLICATION, AUDIO, IMAGE, JAVASCRIPT, JSON, PDF, TEXT, VIDEO},
    Mime,
};

/// Returns `ture` if the content can be displayed inline in the browser.
pub(crate) fn displayed_inline(content_type: &Mime) -> bool {
    let mime_type = content_type.type_();
    if matches!(mime_type, TEXT | IMAGE | AUDIO | VIDEO) {
        true
    } else if mime_type == APPLICATION {
        let mime_subtype = content_type.subtype();
        matches!(mime_subtype, JSON | JAVASCRIPT | PDF) || mime_subtype == "wasm"
    } else {
        false
    }
}

/// Checks whether it has a `content-type: application/json` or similar header.
pub(crate) fn check_json_content_type(content_type: &str) -> bool {
    let essence = if let Some((essence, _)) = content_type.split_once(';') {
        essence
    } else {
        content_type
    };
    essence == "application/json"
        || (essence.starts_with("application/") && essence.ends_with("+json"))
}

/// Gets the data type.
pub(crate) fn get_data_type(content_type: &str) -> &str {
    match content_type {
        "application/json" | "application/problem+json" => "json",
        "application/jsonlines" | "application/x-ndjson" => "ndjson",
        "application/octet-stream" => "bytes",
        "application/x-www-form-urlencoded" => "form",
        "multipart/form-data" => "multipart",
        "text/csv" => "csv",
        "text/plain" => "text",
        _ => {
            if content_type.starts_with("application/") && content_type.ends_with("+json") {
                "json"
            } else {
                content_type
            }
        }
    }
}
