use crate::{
    database::users::{self, Entity as Users},
    utils::{app_error::AppError, jwt::is_valid},
};
use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

pub async fn check_authentication(mut req: Request, next: Next) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());
    let token = if let Some(token) = auth_header {
        token.strip_prefix("Bearer ").unwrap()
    } else {
        return Err(AppError::new(
            StatusCode::UNAUTHORIZED,
            "No token found!".to_owned(),
        ));
    };
    let database = req
        .extensions()
        .get::<DatabaseConnection>()
        .ok_or_else(|| AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Something went wrong!".to_owned(),
        ))?;
    let user = Users::find()
        .filter(users::Column::Token.eq(Some(token)))
        .one(database)
        .await
        .map_err(|error| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    is_valid(token)?;

    let Some(user) = user else {
        return Err(AppError::new(
            StatusCode::UNAUTHORIZED,
            "Something went wrong".to_owned(),
        ));
    };
    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}
