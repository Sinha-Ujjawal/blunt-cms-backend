use dotenv::dotenv;

use actix_cors::Cors;
use serde::Deserialize;

use diesel::prelude::PgConnection;
use diesel::r2d2::ConnectionManager;

pub mod auth;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbPoolConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    host: String,
    port: u16,
    database_url: String,
    jwt_secret: String,
    jwt_expiration_duration: u32,
    redis_server_url: String,
    redis_server_get_connection_timeout: u64,
    cors_allow_all: u8,
}

fn env_var_not_set_msg(env_var: &str) -> String {
    format!(
        "Environment variable `{env_var}` not set",
        env_var = env_var
    )
}

fn env_var_parsing_error_msg(env_var: &str) -> String {
    format!(
        "Environment variable `{env_var}` cannot be parsed",
        env_var = env_var,
    )
}

fn read_from_env<T: std::fmt::Display + std::str::FromStr + std::fmt::Debug>(env_var: &str) -> T {
    match std::env::var(env_var)
        .expect(env_var_not_set_msg(env_var).as_str())
        .parse()
    {
        Ok(parse_value) => parse_value,
        Err(_) => {
            log::error!("{}", env_var_parsing_error_msg(env_var));
            panic!()
        }
    }
}

impl Config {
    pub fn host(&self) -> String {
        self.host.clone()
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn from_env() -> Config {
        dotenv().ok();
        log::info!("Loading configuration");

        let host: String = read_from_env("HOST");
        let port: u16 = read_from_env("PORT");
        let database_url: String = read_from_env("DATABASE_URL");
        let jwt_secret: String = read_from_env("JWT_SECRET");
        let jwt_expiration_duration: u32 = read_from_env("JWT_EXPIRATION_DURATION");
        let redis_server_url: String = read_from_env("REDIS_SERVER_URL");
        let redis_server_get_connection_timeout: u64 =
            read_from_env("REDIS_SERVER_GET_CONNECTION_TIMEOUT");
        let cors_allow_all: u8 = read_from_env("CORS_ALLOW_ALL");

        Config {
            host: host,
            port: port,
            database_url: database_url,
            jwt_secret: jwt_secret,
            jwt_expiration_duration: jwt_expiration_duration,
            redis_server_url: redis_server_url,
            redis_server_get_connection_timeout: redis_server_get_connection_timeout,
            cors_allow_all: cors_allow_all,
        }
    }

    pub fn db_pool(&self) -> DbPool {
        log::info!("Creating database connection pool.");
        // create db connection Pool
        let manager = ConnectionManager::<PgConnection>::new(&self.database_url);
        r2d2::Pool::builder()
            .build(manager) // Aborts if `min_idle` is greater than `max_size`. Need to think about retry
            .expect("Failed to create pool.".as_ref())
    }

    pub fn auth_mgr(&self) -> auth::AuthManager {
        auth::AuthManager::new(
            self.jwt_secret.clone(),
            self.jwt_expiration_duration,
            self.redis_server_url.clone(),
            self.redis_server_get_connection_timeout,
        )
    }

    pub fn cors(&self) -> Cors {
        if self.cors_allow_all == 1 {
            log::info!("Allowing Any Origin");
            Cors::permissive()
        } else {
            log::info!("Default Cors Setup");
            Cors::default()
        }
    }
}
