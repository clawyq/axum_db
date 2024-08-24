mod health;
mod task;

use axum::{routing::get, Extension, Router};

use health::heartbeat;
use sea_orm::DatabaseConnection;
use task::{
    atomic_task_update, create_task, delete_task, get_all_tasks, get_task, partial_task_update,
};

pub async fn create_routes(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/health", get(heartbeat))
        .route("/tasks", get(get_all_tasks).post(create_task))
        .route(
            "/tasks/:task_id",
            get(get_task)
                .delete(delete_task)
                .put(atomic_task_update)
                .patch(partial_task_update),
        )
        .layer(Extension(db))
}
