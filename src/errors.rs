use actix_web::{
    http::{header, StatusCode},
    HttpResponse, HttpResponseBuilder, ResponseError,
};
use deadpool_postgres::PoolError;
use derive_more::Display;
use tokio_pg_mapper::Error as PGMError;
use tokio_postgres::error::Error as PGError;

use crate::models::MessageResponse;

#[derive(Display, Debug)]
pub enum MyError {
    NotFound,
    PGError(PGError),
    PGMError(PGMError),
    PoolError(PoolError),
    #[display(fmt = "Internal Error: {}", _0)]
    InternalError(&'static str),
    #[display(fmt = "Bad Request: {}", _0)]
    BadRequest(&'static str),
    #[display(fmt = "Gateway Timeout: {}", _0)]
    Timeout(&'static str),
}
impl std::error::Error for MyError {}

impl ResponseError for MyError {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .insert_header(header::ContentType::json())
            .json(MessageResponse {
                message: self.to_string(),
            })
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            MyError::BadRequest(_) => StatusCode::BAD_REQUEST,
            MyError::Timeout(_) => StatusCode::GATEWAY_TIMEOUT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
