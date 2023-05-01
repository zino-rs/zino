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
