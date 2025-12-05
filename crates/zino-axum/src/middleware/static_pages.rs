use axum::{body::Body, http::Request, middleware::Next, response::Response};

// Corrects the uri path to achieve the following goals:
//   /pages -> ${public-dir}/index.html
//   /pages/login -> ${public-dir}/login.html
pub(crate) async fn serve_static_pages(mut req: Request<Body>, next: Next) -> Response {
    let uri = req.uri();
    let path = uri.path();
    if let Some((prefix, name)) = path.rsplit_once('/') {
        if !name.contains('.') {
            let mut path_and_query = if prefix.is_empty() || name.is_empty() {
                [path, "/index.html"].concat()
            } else {
                [path, ".html"].concat()
            };
            if let Some(query) = uri.query() {
                path_and_query.push('?');
                path_and_query.push_str(query);
            }
            if let Ok(uri) = path_and_query.parse() {
                *req.uri_mut() = uri;
            }
        }
    }
    next.run(req).await
}
