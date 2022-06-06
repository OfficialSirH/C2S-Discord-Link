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

use crate::handlers::{create_user_pathway, delete_user, update_user};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config = crate::config::Config::new();
    let pool = config.pg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

    let server = HttpServer::new(move || {
        App::new().app_data(Data::new(pool.clone())).service(
            web::scope("/userdata")
                .guard(guard::Header("content-type", "application/json"))
                .service(
                    web::resource("")
                        .guard(guard::fn_guard(|ctx| {
                            ctx.head().headers().contains_key("authorization")
                        }))
                        .wrap(middleware::UserDataAuthorization {})
                        .route(web::post().to(create_user_pathway)),
                )
                .service(delete_user)
                .service(update_user),
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
