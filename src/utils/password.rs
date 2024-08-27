use axum::http::StatusCode;
use bcrypt::{hash, verify, DEFAULT_COST};
use regex::Regex;
use validator::ValidationError;

use super::app_error::AppError;

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.len() < 8 {
        return Err(
            ValidationError::new("length").with_message(std::borrow::Cow::Borrowed(
                "Password needs to be at least 8 characters long.",
            )),
        );
    }

    let special_char = Regex::new(r##"[!@#$%^&*(),.?:{}|<>]"##).unwrap();
    if !special_char.is_match(password) {
        return Err(
            ValidationError::new("special").with_message(std::borrow::Cow::Borrowed(
                "Password must contain at least one special character.",
            )),
        );
    };

    let numbers = Regex::new(r"\d").unwrap();
    if !numbers.is_match(password) {
        return Err(
            ValidationError::new("number").with_message(std::borrow::Cow::Borrowed(
                "Password must contain at least one number.",
            )),
        );
    };

    let uppercase = Regex::new(r"[A-Z]").unwrap();
    if !uppercase.is_match(password) {
        return Err(
            ValidationError::new("case").with_message(std::borrow::Cow::Borrowed(
                "Password must contain at least one uppercase character.",
            )),
        );
    };

    Ok(())
}

pub fn hash_password(password: String) -> Result<String, AppError> {
    hash(password, DEFAULT_COST).map_err(|err| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

pub fn verify_password(password: String, hash: &str) -> Result<bool, AppError> {
    verify(password, hash).map_err(|err| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}
