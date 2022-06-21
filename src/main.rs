pub mod config;
pub mod constants;
pub mod db;
pub mod errors;
mod handlers;
pub mod middleware;
pub mod models;
pub mod role_handling;
pub mod webhook_logging;

use actix_web::{
    guard,
    web::{self, Data},
    App, HttpServer,
};
use deadpool_postgres::Runtime;
use dotenv::dotenv;
use tokio_postgres::NoTls;
use webhook_logging::webhook_log;

use crate::handlers::{create_user, create_user_pathway, delete_user, update_user};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config = crate::config::Config::new();
    let pool = config.pg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(
                web::scope("/userdata")
                    // may have to keep the middleware disabled until the rewritten endpoint is being properly used in-game
                    // temporary alternative will be to have the middleware only used on every other endpoint and have a middleware-like function used for the create endpoint
                    .service(web::resource("").route(web::post().to(create_user_pathway)))
                    .service(
                        web::resource("")
                            .wrap(middleware::UserDataAuthorization {})
                            .route(web::patch().to(update_user))
                            .route(web::delete().to(delete_user)),
                    ),
            )
            .service(
                web::scope("/beta/userdata")
                    .wrap(middleware::UserDataAuthorization {})
                    .service(
                        web::resource("")
                            .guard(guard::Header("content-type", "application/json"))
                            .route(web::patch().to(update_user)),
                    )
                    .route("", web::delete().to(delete_user))
                    .route("", web::post().to(create_user)),
            )
    })
    .bind(config.server_addr.clone())?
    .run();
    webhook_log(
        format!("Server running at http://{}/", config.server_addr),
        constants::LOG::SUCCESSFUL,
    )
    .await;
    println!("Server running at http://{}/", config.server_addr);

    server.await
}
