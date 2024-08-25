use crate::database::users::{self, Entity as Users};
use crate::utils::utils::{hash_password, validate_password};
use axum::{http::StatusCode, Extension, Json};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use serde::{Deserialize, Serialize};
use validator::Validate;

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
) -> Result<Json<UserResponse>, (StatusCode, String)> {
    if let Err(err) = user_req.validate() {
        return Err((StatusCode::BAD_REQUEST, format!("{}", err)));
    }

    let user_model = users::ActiveModel {
        username: Set(user_req.username),
        password: Set(hash_password(user_req.password).unwrap()),
        token: Set(Some("jkngdglkmfd32509i34tsdflml".to_owned())),
        ..Default::default()
    }
    .save(&database)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(Json(UserResponse {
        id: user_model.id.unwrap(),
        username: user_model.username.unwrap(),
        token: user_model.token.unwrap()
    }))
}

pub async fn get_all_users(
    Extension(database): Extension<DatabaseConnection>,
) -> Result<Json<Vec<UserResponse>>, (StatusCode, String)> {
    let user_req = Users::find()
        .all(&database)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
        .into_iter()
        .map(|raw_user| UserResponse {
            id: raw_user.id,
            username: raw_user.username,
            token: raw_user.token
        })
        .collect();
    Ok(Json(user_req))
}
