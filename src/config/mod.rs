use dotenv::dotenv;

use serde::Deserialize;

use diesel::prelude::PgConnection;
use diesel::r2d2::ConnectionManager;

use r2d2_redis::RedisConnectionManager;

pub mod auth;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbPoolConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    database_url: String,
    jwt_secret: String,
    jwt_expiration_duration: u32,
    redis_server_url: String,
    redis_server_get_connection_timeout: u32,
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
    pub fn from_env() -> Config {
        dotenv().ok();
        
        log::info!("Loading configuration");

        let host: String = read_from_env("HOST");
        let port: u16 = read_from_env("PORT");
        let database_url: String = read_from_env("DATABASE_URL");
        let jwt_secret: String = read_from_env("JWT_SECRET");
        let jwt_expiration_duration: u32 = read_from_env("JWT_EXPIRATION_DURATION");
        let redis_server_url: String = read_from_env("REDIS_SERVER_URL");
        let redis_server_get_connection_timeout: u32 =
            read_from_env("REDIS_SERVER_GET_CONNECTION_TIMEOUT");

        Config {
            host: host,
            port: port,
            database_url: database_url,
            jwt_secret: jwt_secret,
            jwt_expiration_duration: jwt_expiration_duration,
            redis_server_url: redis_server_url,
            redis_server_get_connection_timeout: redis_server_get_connection_timeout,
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

    fn redis_pool_result(&self) -> Result<auth::RedisPool, r2d2::Error> {
        log::info!("Creating Redis connection pool.");
        // create redis connection pook
        let manager = RedisConnectionManager::new(self.redis_server_url.clone()).unwrap();
        r2d2::Pool::builder()
            .connection_timeout(std::time::Duration::from_secs(
                self.redis_server_get_connection_timeout.into(),
            ))
            .build(manager) // Aborts if `min_idle` is greater than `max_size`. Need to think about retry
    }

    pub fn auth_mgr(&self) -> auth::AuthManager {
        let jwt_auth_mgr =
            auth::JWTAuthManager::new(self.jwt_secret.clone(), self.jwt_expiration_duration);
        match self.redis_pool_result() {
            Ok(redis_pool) => auth::AuthManager::RedisAuthManager(jwt_auth_mgr, redis_pool),
            Err(_) => {
                log::error!("Failed connecting to redis, fallback to simple-jwt-auth-manager");                
                auth::AuthManager::SimpleAuthManager(jwt_auth_mgr)
            },
        }
    }

    pub fn address(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}
