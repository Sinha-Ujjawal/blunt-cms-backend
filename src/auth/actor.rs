use crate::auth::simple_jwt_helper::{Claims, SimpleJWT};
use actix::{Actor, Handler, Message, SyncContext};
use jsonwebtoken as jwt;
use jsonwebtoken::TokenData;
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;

use r2d2_redis::redis::{Commands, RedisError, ToRedisArgs};
use r2d2_redis::RedisConnectionManager;

pub type RedisPool = r2d2::Pool<RedisConnectionManager>;
pub type RedisPoolConnection = r2d2::PooledConnection<RedisConnectionManager>;

#[derive(std::clone::Clone)]
pub enum AuthManager {
    SimpleAuthManager(SimpleJWT),
    RedisAuthManager(SimpleJWT, RedisPool),
}

impl AuthManager {
    fn redis_pool_result(
        redis_server_url: String,
        redis_server_get_connection_timeout: u64,
    ) -> Result<RedisPool, r2d2::Error> {
        log::info!("Creating Redis connection pool.");
        // create redis connection pook
        let manager = RedisConnectionManager::new(redis_server_url).unwrap();
        r2d2::Pool::builder()
            .connection_timeout(std::time::Duration::from_secs(
                redis_server_get_connection_timeout,
            ))
            .build(manager) // Aborts if `min_idle` is greater than `max_size`. Need to think about retry
    }

    pub fn new(
        jwt_secret: String,
        expiration_duration: u32,
        redis_server_url: String,
        redis_server_get_connection_timeout: u64,
    ) -> Self {
        let jwt_auth_mgr = SimpleJWT::new(jwt_secret, expiration_duration);
        match Self::redis_pool_result(redis_server_url, redis_server_get_connection_timeout) {
            Ok(redis_pool) => AuthManager::RedisAuthManager(jwt_auth_mgr, redis_pool),
            Err(_) => {
                log::error!("Failed connecting to redis, fallback to simple-jwt-auth-manager");
                AuthManager::SimpleAuthManager(jwt_auth_mgr)
            }
        }
    }

    fn cache_token<T: Serialize + ToRedisArgs>(
        conn: &mut RedisPoolConnection,
        data: T,
        token: String,
    ) -> Result<String, RedisError> {
        conn.set(data, token)
    }

    fn get_token_from_cache<T: Serialize + ToRedisArgs>(
        conn: &mut RedisPoolConnection,
        data: T,
    ) -> Result<String, RedisError> {
        conn.get(data)
    }

    fn create_new_token_with_cache<T: Serialize + ToRedisArgs + Copy>(
        jwt_auth_mgr: &SimpleJWT,
        db_redis: &RedisPool,
        data: T,
    ) -> Option<String> {
        let token = jwt_auth_mgr.create_token::<T>(data)?;

        match db_redis.get() {
            Err(_) => {}
            Ok(mut conn) => {
                let _ = Self::cache_token(&mut conn, data, token.clone());
            }
        };

        Some(token)
    }

    pub fn create_token<T: Serialize + DeserializeOwned + ToRedisArgs + Copy>(
        &self,
        data: T,
    ) -> Option<String> {
        use AuthManager::*;
        match self {
            SimpleAuthManager(jwt_auth_mgr) => jwt_auth_mgr.create_token::<T>(data),
            RedisAuthManager(jwt_auth_mgr, db_redis) => match db_redis.get() {
                Err(_) => jwt_auth_mgr.create_token::<T>(data),
                Ok(mut conn) => match Self::get_token_from_cache(&mut conn, data) {
                    Ok(token) if jwt_auth_mgr.validate_token::<T>(&token) => Some(token),
                    _ => Self::create_new_token_with_cache(jwt_auth_mgr, db_redis, data),
                },
            },
        }
    }

    fn decode_token<'a, T: DeserializeOwned>(
        &self,
        token: &'a str,
    ) -> jwt::errors::Result<TokenData<Claims<T>>> {
        use AuthManager::*;
        match self {
            SimpleAuthManager(jwt_auth_mgr) => jwt_auth_mgr.decode_token::<Claims<T>>(token),
            RedisAuthManager(jwt_auth_mgr, _) => jwt_auth_mgr.decode_token::<Claims<T>>(token),
        }
    }

    pub fn extract_claim<'a, T: DeserializeOwned>(
        &self,
        token: &'a str,
    ) -> jwt::errors::Result<T> {
        let token_data = self.decode_token::<T>(token)?;
        Ok(token_data.claims.data)
    }
}

impl Actor for AuthManager {
    type Context = SyncContext<Self>;
}

#[derive(Message)]
#[rtype(result = "Option<String>")]
pub struct CreateToken<T: Serialize + DeserializeOwned + ToRedisArgs + std::fmt::Debug + Copy> {
    pub data: T,
}

impl<T: Serialize + DeserializeOwned + ToRedisArgs + std::fmt::Debug + Copy> Handler<CreateToken<T>>
    for AuthManager
{
    type Result = Option<String>;

    fn handle(&mut self, msg: CreateToken<T>, _: &mut Self::Context) -> Self::Result {
        self.create_token(msg.data)
    }
}

#[derive(Message)]
#[rtype(result = "jwt::errors::Result<T>")]
pub struct ExtractClaim<T: 'static + DeserializeOwned + std::fmt::Debug> {
    pub token: String,
    pub phantom: PhantomData<T>,
}

impl<T: DeserializeOwned + std::fmt::Debug> Handler<ExtractClaim<T>> for AuthManager {
    type Result = jwt::errors::Result<T>;

    fn handle(&mut self, msg: ExtractClaim<T>, _: &mut Self::Context) -> Self::Result {
        self.extract_claim(&msg.token)
    }
}
