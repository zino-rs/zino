use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use std::{
    future::{ready, Future, Ready},
    pin::Pin,
};
use zino_core::request::RequestContext;

#[derive(Default)]
pub struct RequestContextInitializer;

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

pub struct RequestContextMiddleware<S> {
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
            req.extensions_mut().insert(ctx);
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
