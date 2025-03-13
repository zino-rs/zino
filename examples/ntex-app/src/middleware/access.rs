use ntex::{
    service::{Middleware, Service, ServiceCtx},
    web::{Error, ErrorRenderer, WebRequest, WebResponse},
};
use zino::{Request, prelude::*};

#[derive(Default)]
pub struct UserSessionInitializer;

pub struct UserSessionMiddleware<S> {
    service: S,
}

impl<S> Middleware<S> for UserSessionInitializer {
    type Service = UserSessionMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        UserSessionMiddleware { service }
    }
}

impl<S, Err> Service<WebRequest<Err>> for UserSessionMiddleware<S>
where
    S: Service<WebRequest<Err>, Response = WebResponse, Error = Error>,
    Err: ErrorRenderer,
{
    type Response = WebResponse;
    type Error = Error;

    ntex::forward_ready!(service);

    async fn call(
        &self,
        req: WebRequest<Err>,
        ctx: ServiceCtx<'_, Self>,
    ) -> Result<Self::Response, Self::Error> {
        let mut req = Request::from(req);
        match req.parse_jwt_claims(JwtClaims::shared_key()) {
            Ok(claims) => {
                if let Ok(session) = UserSession::<Uuid>::try_from_jwt_claims(claims) {
                    req.set_data(session);
                } else {
                    let message = "401 Unauthorized: invalid JWT claims";
                    let rejection = Rejection::with_message(message).context(&req).into();
                    let result: zino::Result<Self::Response> = Err(rejection);
                    return result.map_err(|err| err.into());
                }
            }
            Err(rejection) => {
                let result: zino::Result<Self::Response> = Err(rejection.into());
                return result.map_err(|err| err.into());
            }
        }

        let req = WebRequest::try_from(req)?;
        let res = ctx.call(&self.service, req).await?;
        Ok(res)
    }
}
