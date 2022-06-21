use std::{
    future::{ready, Future, Ready},
    pin::Pin,
};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};

trait InvalidItems<T> {
    fn invalid_auth(self) -> Result<T, Error>;

    fn invalid_header(self) -> Result<T, Error>;
}

impl<T> InvalidItems<T> for Option<T> {
    fn invalid_auth(self) -> Result<T, Error> {
        self.ok_or(Error::from(actix_web::error::ErrorBadRequest(
            "Invalid authorization header",
        )))
    }

    fn invalid_header(self) -> Result<T, Error> {
        self.ok_or(Error::from(actix_web::error::ErrorBadRequest(
            "Invalid header",
        )))
    }
}

impl<T, E> InvalidItems<T> for Result<T, E> {
    fn invalid_auth(self) -> Result<T, Error> {
        self.or_else(|_| {
            Err(Error::from(actix_web::error::ErrorBadRequest(
                "Invalid authorization header",
            )))
        })
    }

    fn invalid_header(self) -> Result<T, Error> {
        self.or_else(|_| {
            Err(Error::from(actix_web::error::ErrorBadRequest(
                "Invalid header",
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
        let mut distribution_header = headers.remove("X-Distribution-Channel");

        let auth_header = auth_header.next();
        let distribution_header = distribution_header.next();

        let path = req.path().to_owned();

        let fut = self.service.call(req);
        Box::pin(async move {
            // obtain the auth header and convert to string
            let auth_header = auth_header.invalid_auth()?;
            let auth_header = auth_header.to_str().invalid_auth()?;

            // split auth header from "Basic {auth}"
            let auth_header = auth_header.split_whitespace().collect::<Vec<_>>();

            // check if auth header is "Basic"
            if *auth_header.get(0).invalid_auth()? != "Basic" {
                return Err(Error::from(actix_web::error::ErrorBadRequest(
                    "Invalid authorization header",
                )));
            }

            // obtain the base64 string from the header
            let auth_header = auth_header.get(1).invalid_auth()?;
            // decode base64 from the string
            let auth_header = base64::decode(auth_header).invalid_auth()?;

            // convert the bytes to a string
            let auth_header = String::from_utf8(auth_header).invalid_auth()?;

            println!("testing basic auth header decoding: {}", auth_header);

            // get the username and password from the email:player_token
            let auth_header = auth_header.split(":").collect::<Vec<_>>();
            let player_email = auth_header.get(0).invalid_auth()?;
            let player_token = auth_header.get(1).invalid_auth()?;

            println!("id: {}, token: {}", player_email, player_token);

            // TODO: put together validation, replace player_email with email to be able to perform proper validation, and require beta bool so then we know which server to validate against

            let config = crate::config::Config::new();
            // just a little reminder here for what URLs to be used when later implementing reqwest
            // config.game_saves_dev_api;
            // config.game_saves_prod_api;

            let distribution_header = distribution_header.invalid_header()?;
            let distribution_header = distribution_header.to_str().invalid_header()?;

            // use reqwest to make a post to either the dev or prod api depending on the route or header
            // if the route is /beta/userdata or the header X-Distribution-Channel is "Beta" then set string url to dev api
            // if the route is /userdata or the header X-Distribution-Channel doesn't exist then set string url to prod api
            let url = if path == "/beta/userdata" || distribution_header == "Beta" {
                config.game_saves_dev_api
            } else {
                config.game_saves_prod_api
            };

            println!(
                "path: {}, header: {}, url: {}",
                path, distribution_header, url
            );

            // let mut response = reqwest::Client::new()
            //     .post(&config.game_saves_dev_api)
            //     .body(format!("{}", player_email))
            //     .send()
            //     .await
            //     .invalid_auth()?;

            let res = fut.await?;

            Ok(res)
        })
    }
}
