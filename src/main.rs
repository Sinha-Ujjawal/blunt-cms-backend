// dependencies
#[macro_use]
extern crate diesel;
use crate::config::Config;
use actix_web::{middleware, web::Data, App, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

// module declarations
mod config;
mod models;
mod schema;
mod selectors;
mod services;
mod views;

// type declarations
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::from_env();

    let pool = config.db_bool();

    println!("Started server on: {:?}", config.address());

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool.clone()))
            .wrap(middleware::Logger::default()) // enable logger
            .service(views::users::signup)
    })
    .bind((config.host, config.port))?
    .run()
    .await?;

    Ok(())
}
