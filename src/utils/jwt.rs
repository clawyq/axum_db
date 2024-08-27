use std::env;

use chrono::{Duration, Utc};
use http::StatusCode;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    exp: usize,
    iat: usize,
}

pub fn create_jwt() -> Result<String, (StatusCode, String)> {
    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::seconds(30)).timestamp() as usize;
    let claim = Claims { exp, iat };
    let secret = env::var("JWT_SECRET").expect("JWT secret not set");
    let key = EncodingKey::from_secret(secret.as_bytes());
    encode(&Header::default(), &claim, &key)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))
}

pub fn is_valid(token: &str) -> Result<bool, (StatusCode, String)> {
    let secret = env::var("JWT_SECRET").expect("JWT secret not set");
    let key = DecodingKey::from_secret(secret.as_bytes());
    decode::<Claims>(token, &key, &Validation::new(Algorithm::HS256))
        .map_err(|error| {
            match error.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => (StatusCode::UNAUTHORIZED, "Invalid token.".to_owned()),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
            }
    })?;
    Ok(true)
}
