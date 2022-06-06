use std::{
    future::{ready, Future, Ready},
    pin::Pin,
};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};

trait InvalidAuth<T> {
    fn invalid_auth(self) -> Result<T, Error>;
}

impl<T> InvalidAuth<T> for Option<T> {
    fn invalid_auth(self) -> Result<T, Error> {
        self.ok_or(Error::from(actix_web::error::ErrorBadRequest(
            "Invalid authorization header",
        )))
    }
}

impl<T, E> InvalidAuth<T> for Result<T, E> {
    fn invalid_auth(self) -> Result<T, Error> {
        self.or_else(|_| {
            Err(Error::from(actix_web::error::ErrorBadRequest(
                "Invalid authorization header",
            )))
        })
    }
}

type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct UserDataAuthorization;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for UserDataAuthorization
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = UserDataAuthorizationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(UserDataAuthorizationMiddleware { service }))
    }
}

pub struct UserDataAuthorizationMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for UserDataAuthorizationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if req.query_string() == "" {
            let fut = self.service.call(req);

            return Box::pin(async move {
                let res = fut.await?;

                Ok(res)
            });
        }

        let headers = &mut req.headers().clone();

        let mut auth_header = headers.remove("authorization");

        let auth_header = auth_header.next();

        let fut = self.service.call(req);
        Box::pin(async move {
            // obtain the auth header and convert to string
            let auth_header = auth_header.invalid_auth()?;
            let auth_header = auth_header.to_str().invalid_auth()?;

            // split auth header from "Basic {auth}"
            let auth_header = auth_header.split_whitespace().collect::<Vec<_>>();

            // obtain the base64 string from the header
            let auth_header = auth_header.get(1).invalid_auth()?;
            // decode base64 from the string
            let auth_header = base64::decode(auth_header).invalid_auth()?;

            // convert the bytes to a string
            let auth_header = String::from_utf8(auth_header).invalid_auth()?;

            println!("testing basic auth header decoding: {}", auth_header);

            // get the username and password from the player_id:player_token
            let auth_header = auth_header.split(":").collect::<Vec<_>>();
            let player_id = auth_header.get(0).invalid_auth()?;
            let player_token = auth_header.get(1).invalid_auth()?;

            println!("id: {}, token: {}", player_id, player_token);

            // TODO: put together validation, replace player_id with email to be able to perform proper validation, and require beta bool so then we know which server to validate against

            let config = crate::config::Config::new();
            // just a little reminder here for what URLs to be used when later implementing reqwest
            config.game_saves_dev_api;
            config.game_saves_prod_api;

            let res = fut.await?;

            Ok(res)
        })
    }
}
