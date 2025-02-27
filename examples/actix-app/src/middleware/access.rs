use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use std::{
    future::{Future, Ready, ready},
    pin::Pin,
};
use zino::{Request, prelude::*};

#[derive(Default)]
pub struct UserSessionInitializer;

impl<S, B> Transform<S, ServiceRequest> for UserSessionInitializer
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = UserSessionMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(UserSessionMiddleware { service }))
    }
}

pub struct UserSessionMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for UserSessionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let mut req = Request::from(req);
        match req.parse_jwt_claims(JwtClaims::shared_key()) {
            Ok(claims) => {
                if let Ok(session) = UserSession::<Uuid>::try_from_jwt_claims(claims) {
                    req.set_data(session);
                } else {
                    return Box::pin(async move {
                        let message = "401 Unauthorized: invalid JWT claims";
                        let rejection = Rejection::with_message(message).context(&req).into();
                        let result: zino::Result<Self::Response> = Err(rejection);
                        result.map_err(|err| err.into())
                    });
                }
            }
            Err(rejection) => {
                return Box::pin(async move {
                    let result: zino::Result<Self::Response> = Err(rejection.into());
                    result.map_err(|err| err.into())
                });
            }
        }

        let req = ServiceRequest::from(req);
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
