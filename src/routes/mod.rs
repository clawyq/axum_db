mod guard;
mod health;
mod task;
mod user;

use axum::{
    extract::FromRef, middleware, routing::{get, post}, Router
};

use guard::check_authentication;
use health::heartbeat;
use sea_orm::DatabaseConnection;
use task::{
    atomic_task_update, create_task, delete_task, get_all_tasks, get_task, partial_task_update,
};
use user::{create_user, get_all_users, login, logout};

#[derive(Clone, FromRef)]
pub struct AppState {
    pub database: DatabaseConnection
}

pub async fn create_routes(database: DatabaseConnection) -> Router {
    let app_state = AppState { database };
    Router::new()
        .route("/health", get(heartbeat))
        .route("/logout", post(logout))
        .route_layer(middleware::from_fn_with_state(app_state.clone(), check_authentication))
        .route("/login", post(login))
        .route("/users", get(get_all_users).post(create_user))
        .route("/tasks", get(get_all_tasks).post(create_task))
        .route(
            "/tasks/:task_id", // this can be middlewared
            get(get_task)
                .delete(delete_task)
                .put(atomic_task_update)
                .patch(partial_task_update),
        )
        .with_state(app_state)
}
