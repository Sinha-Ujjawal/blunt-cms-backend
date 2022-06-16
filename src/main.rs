#[macro_use]
extern crate diesel;
use crate::config::Config;
use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use dotenv::dotenv;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod config;
mod errors;
mod models;
mod openapi;
mod schema;
mod selectors;
mod services;
mod utils;
mod views;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = Config::from_env();
    let db_pool = config.db_pool();
    let auth_mgr = config.auth_mgr();
    let host = config.host();
    let port = config.port();

    log::info!("Started server on: http://{}:{}", host, port);

    HttpServer::new(move || {
        let cors = config.cors();
        
        App::new()
            .wrap(cors)
            .app_data(Data::new(db_pool.clone()))
            .app_data(Data::new(auth_mgr.clone()))
            .wrap(Logger::default()) // enable logger
            .configure(views::users::config)
            .configure(views::posts::config)
            .configure(views::drafts::config)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-doc/openapi.json", views::users::ApiDoc::openapi()),
            )
    })
    .bind((host, port))?
    .run()
    .await?;

    Ok(())
}
