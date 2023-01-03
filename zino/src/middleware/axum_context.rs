use crate::AxumExtractor;
use axum::{
    body::Body,
    http::{Request, Response},
};
use futures::future::BoxFuture;
use std::{mem, task};
use tower::Service;
use zino_core::RequestContext;

/// Request context middleware.
#[derive(Debug, Clone)]
pub(crate) struct ContextMiddleware<S> {
    inner: S,
}

impl<S> ContextMiddleware<S> {
    /// Creates a new instance.
    #[inline]
    pub(crate) fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S, ResBody> Service<Request<Body>> for ContextMiddleware<S>
where
    S: Service<Request<Body>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let cloned_inner = self.inner.clone();
        let mut inner = mem::replace(&mut self.inner, cloned_inner);
        Box::pin(async move {
            let mut req_extractor = AxumExtractor(req);
            let ext = match req_extractor.get_context() {
                Some(_) => None,
                None => {
                    let mut ctx = req_extractor.new_context();
                    let original_uri = req_extractor.original_uri().await;
                    ctx.set_request_path(original_uri.path());
                    Some(ctx)
                }
            };

            let mut req = req_extractor.0;
            if let Some(ctx) = ext {
                req.extensions_mut().insert(ctx);
            }
            inner.call(req).await
        })
    }
}
