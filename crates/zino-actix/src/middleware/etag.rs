use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    http::{
        StatusCode,
        header::{ETAG, IF_NONE_MATCH},
    },
};
use std::{
    future::{Future, Ready, ready},
    pin::Pin,
};

#[derive(Default)]
pub(crate) struct ETagFinalizer;

impl<S, B> Transform<S, ServiceRequest> for ETagFinalizer
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ETagMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ETagMiddleware { service }))
    }
}

pub(crate) struct ETagMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ETagMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if req.method().is_idempotent() {
            let req_etag = req.headers().get(IF_NONE_MATCH).cloned();
            let fut = self.service.call(req);
            Box::pin(async move {
                let mut res = fut.await?;
                if let Some(etag) = res.headers_mut().remove("x-etag").next() {
                    if req_etag.as_ref() == Some(&etag) && res.status().is_success() {
                        *res.response_mut().status_mut() = StatusCode::NOT_MODIFIED;
                    }
                    res.headers_mut().insert(ETAG, etag);
                }
                Ok(res)
            })
        } else {
            let fut = self.service.call(req);
            Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            })
        }
    }
}
