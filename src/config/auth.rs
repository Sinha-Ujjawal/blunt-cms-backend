use chrono::offset::Utc;
use jsonwebtoken as jwt;
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(std::clone::Clone)]
pub struct AuthManager {
    jwt_secret: String,
    expiration_duration: u32,
    algorithm: Algorithm,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims<T> {
    data: T,
    exp: usize,
}

impl AuthManager {
    pub fn new(jwt_secret: String, expiration_duration: u32) -> Self {
        AuthManager {
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
            Err(_err) => false
        }
    }
}
