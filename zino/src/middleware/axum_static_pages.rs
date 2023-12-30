use axum::{body::Body, http::Request, middleware::Next, response::Response};

// Corrects the uri path to achieve the following goals:
//   /page -> ${page-dir}/index.html
//   /page/login -> ${page-dir}/login.html
pub(crate) async fn serve_static_pages(mut req: Request<Body>, next: Next<Body>) -> Response {
    let uri = req.uri();
    let path = uri.path();
    if let Some((_, name)) = path.rsplit_once('/') {
        if !(name.contains('.') || name.is_empty()) {
            let mut path_and_query = if name == "page" {
                format!("{path}/index.html")
            } else {
                format!("{path}.html")
            };
            if let Some(query) = uri.query() {
                path_and_query.push_str("?");
                path_and_query.push_str(query);
            }
            if let Ok(uri) = path_and_query.parse() {
                *req.uri_mut() = uri;
            }
        }
    }
    next.run(req).await
}
