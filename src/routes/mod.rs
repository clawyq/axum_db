mod health;

use axum::{
    routing::get, Router
};

use health::heartbeat;
use sea_orm::DatabaseConnection;

pub async fn create_routes(db: DatabaseConnection) -> Router {
    Router::new()
        .route("health", get(heartbeat))
}
