use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use std::{
    future::{Future, Ready, ready},
    pin::Pin,
    sync::Arc,
};
use tracing::Span;
use zino_http::request::RequestContext;

#[derive(Default)]
pub(crate) struct RequestContextInitializer;

impl<S, B> Transform<S, ServiceRequest> for RequestContextInitializer
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestContextMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestContextMiddleware { service }))
    }
}

pub(crate) struct RequestContextMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestContextMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let req = crate::Request::from(req);
        let new_context = req.get_context().is_none().then(|| req.new_context());

        let req = ServiceRequest::from(req);
        if let Some(ctx) = new_context {
            Span::current().record("context.request_id", ctx.request_id().to_string());
            req.extensions_mut().insert(Arc::new(ctx));
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
