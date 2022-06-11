use chrono::offset::Utc;
use jsonwebtoken as jwt;
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use r2d2_redis::redis::{Commands, RedisError, ToRedisArgs};
use r2d2_redis::RedisConnectionManager;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub type RedisPool = r2d2::Pool<RedisConnectionManager>;
pub type RedisPoolConnection = r2d2::PooledConnection<RedisConnectionManager>;

#[derive(Clone)]
pub struct JWTManager {
    jwt_secret: String,
    expiration_duration: u32,
    algorithm: Algorithm,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims<T> {
    data: T,
    exp: usize,
}

impl JWTManager {
    pub fn new(jwt_secret: String, expiration_duration: u32) -> Self {
        JWTManager {
            jwt_secret: jwt_secret,
            expiration_duration: expiration_duration,
            algorithm: Algorithm::HS256,
        }
    }

    pub fn create_token<T: Serialize>(&self, data: T) -> Option<String> {
        let expiration = Utc::now()
            .checked_add_signed(chrono::Duration::seconds(self.expiration_duration as i64))
            .expect("Valid Timestamp")
            .timestamp();
        let claims = Claims {
            data: data,
            exp: expiration as usize,
        };

        let header = Header::new(self.algorithm);
        match encode::<Claims<T>>(
            &header,
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        ) {
            Ok(token) => Some(token),
            Err(_) => None,
        }
    }

    fn decode_token<T: DeserializeOwned>(
        &self,
        token: String,
    ) -> jwt::errors::Result<TokenData<T>> {
        let validation = Validation::new(self.algorithm);
        decode::<T>(
            &token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation,
        )
    }

    pub fn validate_token<T: DeserializeOwned + std::fmt::Debug>(&self, token: String) -> bool {
        match self.decode_token::<Claims<T>>(token) {
            Ok(_token_message) => true,
            Err(_err) => false,
        }
    }
}

#[derive(std::clone::Clone)]
pub enum AuthManager {
    SimpleAuthManager(JWTManager),
    RedisAuthManager(JWTManager, RedisPool),
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
        let jwt_auth_mgr = JWTManager::new(jwt_secret, expiration_duration);
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
        jwt_auth_mgr: &JWTManager,
        db_redis: &RedisPool,
        data: T,
    ) -> Option<String> {
        match db_redis.get() {
            Err(_) => jwt_auth_mgr.create_token::<T>(data),
            Ok(mut conn) => match jwt_auth_mgr.create_token::<T>(data) {
                None => None,
                Some(token) => {
                    let _ = Self::cache_token(&mut conn, data, token.clone());
                    Some(token)
                }
            },
        }
    }

    pub fn create_token<T: Serialize + DeserializeOwned + ToRedisArgs + std::fmt::Debug + Copy>(
        &self,
        data: T,
    ) -> Option<String> {
        use AuthManager::*;
        match self {
            SimpleAuthManager(jwt_auth_mgr) => jwt_auth_mgr.create_token::<T>(data),
            RedisAuthManager(jwt_auth_mgr, db_redis) => match db_redis.get() {
                Err(_) => jwt_auth_mgr.create_token::<T>(data),
                Ok(mut conn) => match Self::get_token_from_cache(&mut conn, data) {
                    Ok(token) if jwt_auth_mgr.validate_token::<T>(token.clone()) => Some(token),
                    _ => Self::create_new_token_with_cache(jwt_auth_mgr, db_redis, data),
                },
            },
        }
    }

    pub fn validate_token<T: DeserializeOwned + std::fmt::Debug>(&self, token: String) -> bool {
        use AuthManager::*;
        match self {
            SimpleAuthManager(jwt_auth_mgr) => jwt_auth_mgr.validate_token::<T>(token),
            RedisAuthManager(jwt_auth_mgr, _) => jwt_auth_mgr.validate_token::<T>(token),
        }
    }
}
