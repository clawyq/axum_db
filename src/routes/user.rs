use axum::{http::StatusCode, Extension, Json};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    Set,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::database::users::{self, Entity as Users, Model};
use crate::utils::app_error::AppError;
use crate::utils::jwt::create_jwt;
use crate::utils::password::{hash_password, validate_password, verify_password};

#[derive(Serialize)]
pub struct UserResponse {
    id: i32,
    username: String,
    token: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct UserRequest {
    #[validate(email)]
    username: String,
    #[validate(custom(function=validate_password))]
    password: String,
}

pub async fn create_user(
    Extension(database): Extension<DatabaseConnection>,
    Json(user_req): Json<UserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    if let Err(err) = user_req.validate() {
        return Err(AppError::new(StatusCode::BAD_REQUEST, format!("{}", err)));
    }

    let user_model = users::ActiveModel {
        username: Set(user_req.username),
        password: Set(hash_password(user_req.password).unwrap()),
        token: Set(Some(create_jwt()?)),
        ..Default::default()
    }
    .save(&database)
    .await
    .map_err(|err| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(Json(UserResponse {
        id: user_model.id.unwrap(),
        username: user_model.username.unwrap(),
        token: user_model.token.unwrap(),
    }))
}

pub async fn get_all_users(
    Extension(database): Extension<DatabaseConnection>,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    let user_req = Users::find()
        .all(&database)
        .await
        .map_err(|err| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
        .into_iter()
        .map(|raw_user| UserResponse {
            id: raw_user.id,
            username: raw_user.username,
            token: raw_user.token,
        })
        .collect();
    Ok(Json(user_req))
}

pub async fn login(
    Extension(database): Extension<DatabaseConnection>,
    Json(user_req): Json<UserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    if user_req.username.is_empty() || user_req.password.is_empty() {
        return Err(AppError::new(
            StatusCode::BAD_REQUEST,
            "Please enter all login details.".to_owned(),
        ));
    }
    let user = Users::find()
        .filter(users::Column::Username.eq(user_req.username))
        .one(&database)
        .await
        .map_err(|error| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    if let None = user {
        return Err(AppError::new(StatusCode::NOT_FOUND, "Username not found.".to_owned()));
    }

    let user = user.unwrap();
    if !verify_password(user_req.password, &user.password[..])? {
        return Err(AppError::new(StatusCode::UNAUTHORIZED, "Wrong credentials.".to_owned()));
    }
    let mut user = user.into_active_model();
    user.token = Set(Some(create_jwt()?));
    let user = user
        .save(&database)
        .await
        .map_err(|error| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    Ok(Json(UserResponse {
        id: user.id.unwrap(),
        username: user.username.unwrap(),
        token: user.token.unwrap(),
    }))
}

pub async fn logout(
    Extension(database): Extension<DatabaseConnection>,
    Extension(user): Extension<Model>,
) -> Result<(), AppError> {
    let mut user = user.into_active_model();

    user.token = Set(None);
    user.save(&database)
        .await
        .map_err(|error| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    Ok(())
}
