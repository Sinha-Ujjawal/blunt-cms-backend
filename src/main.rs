// dependencies
// #[macro_use]
extern crate diesel;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use actix_web::{web::Data, App, middleware, HttpServer};

// module declarations
mod services;
mod models;

// type declarations
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // create db connection Pool
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    log::info!("starting HTTP server at http://localhost:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool.clone()))
            .wrap(middleware::Logger::default()) // enable logger
            .service(services::signup)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
