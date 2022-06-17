use chrono::offset::Utc;
use jsonwebtoken as jwt;
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Clone)]
pub struct SimpleJWT {
    jwt_secret: String,
    expiration_duration: u32,
    algorithm: Algorithm,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims<T> {
    pub data: T,
    exp: usize,
}

impl SimpleJWT {
    pub fn new(jwt_secret: String, expiration_duration: u32) -> Self {
        SimpleJWT {
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
        encode::<Claims<T>>(
            &header,
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .ok()
    }

    pub fn decode_token<'a, T: DeserializeOwned>(
        &self,
        token: &'a str,
    ) -> jwt::errors::Result<TokenData<T>> {
        let validation = Validation::new(self.algorithm);
        decode::<T>(
            &token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation,
        )
    }

    pub fn validate_token<'a, T: DeserializeOwned>(
        &self,
        token: &'a str,
    ) -> bool {
        match self.decode_token::<Claims<T>>(token) {
            Ok(_token_message) => true,
            Err(_err) => false,
        }
    }
}
