use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use chrono::{DateTime, FixedOffset};
use sea_orm::{
    prelude::DateTimeWithTimeZone, ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection,
    EntityTrait, IntoActiveModel, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};

use crate::database::{
    tasks::{self, Entity as Tasks},
    users::{self, Entity as Users},
};

#[derive(Deserialize)]
pub struct TaskRequest {
    id: Option<i32>,
    priority: Option<String>,
    title: Option<String>,
    completed_at: Option<DateTimeWithTimeZone>,
    description: Option<String>,
    deleted_at: Option<DateTimeWithTimeZone>,
    user_id: Option<i32>,
    is_default: Option<bool>,
}

#[derive(Serialize)]
pub struct TaskResponse {
    id: Option<i32>,
    title: String,
    description: Option<String>,
    priority: Option<String>,
    deleted_at: Option<DateTime<FixedOffset>>,
    user_id: Option<i32>,
}

impl IntoResponse for TaskResponse {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

#[derive(Deserialize)]
pub struct TaskQueryParams {
    title: Option<String>,
    priority: Option<String>,
}

#[derive(Deserialize)]
pub struct DeleteParams {
    soft: Option<bool>,
}

pub async fn create_task(
    State(database): State<DatabaseConnection>,
    authorisation: TypedHeader<Authorization<Bearer>>,
    Json(req): Json<TaskRequest>,
) -> Result<(StatusCode, TaskResponse), (StatusCode, String)> {
    let token = authorisation.token();
    let user = if let Some(user) = Users::find()
        .filter(users::Column::Token.eq(token))
        .one(&database)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?
    {
        user
    } else {
        return Err((StatusCode::UNAUTHORIZED, "Action not allowed.".to_owned()));
    };

    if let None = req.title {
        return Err((StatusCode::BAD_REQUEST, "Title is required.".to_owned()));
    }

    let task = tasks::ActiveModel {
        title: Set(req.title.unwrap()),
        description: Set(req.description),
        priority: Set(req.priority),
        user_id: Set(Some(user.id)),
        ..Default::default()
    };

    match task.save(&database).await {
        Ok(saved_task) => Ok((
            StatusCode::CREATED,
            TaskResponse {
                id: Some(saved_task.id.unwrap()),
                title: saved_task.title.unwrap(),
                description: saved_task.description.unwrap(),
                priority: saved_task.priority.unwrap(),
                deleted_at: saved_task.deleted_at.unwrap(),
                user_id: Some(user.id),
            },
        )),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    }
}

/**
 * todo paginate
 * map_err here to show diff (more idiomatic) way of err handling
 */
pub async fn get_all_tasks(
    State(database): State<DatabaseConnection>,
    Query(query_params): Query<TaskQueryParams>,
) -> Result<Json<Vec<TaskResponse>>, StatusCode> {
    let conditions = parse_query_params_into_conditions(query_params);
    let tasks = Tasks::find()
        .filter(conditions)
        .all(&database)
        .await
        .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|task| TaskResponse {
            id: Some(task.id),
            title: task.title,
            description: task.description,
            priority: task.priority,
            deleted_at: task.deleted_at,
            user_id: task.user_id,
        })
        .collect();
    Ok(Json(tasks))
}

pub async fn get_task(
    Path(task_id): Path<i32>,
    State(database): State<DatabaseConnection>,
) -> Result<TaskResponse, StatusCode> {
    let db_req = Tasks::find_by_id(task_id)
        .filter(tasks::Column::DeletedAt.is_null())
        .one(&database)
        .await;
    if let Err(error) = db_req {
        eprintln!("{error}");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    match db_req.unwrap() {
        Some(task) => Ok(TaskResponse {
            id: Some(task.id),
            title: task.title,
            description: task.description,
            priority: task.priority,
            deleted_at: task.deleted_at,
            user_id: task.user_id,
        }),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn atomic_task_update(
    Path(task_id): Path<i32>,
    State(database): State<DatabaseConnection>,
    Json(req): Json<TaskRequest>,
) -> Result<(), (StatusCode, String)> {
    if let None = req.title {
        return Err((StatusCode::BAD_REQUEST, "Title is required.".to_owned()));
    }

    let concrete_task = tasks::ActiveModel {
        id: Set(task_id),
        priority: Set(req.priority),
        title: Set(req.title.unwrap()),
        completed_at: Set(req.completed_at),
        description: Set(req.description),
        deleted_at: Set(req.deleted_at),
        user_id: Set(req.user_id),
        is_default: Set(req.is_default),
    };

    let _ = Tasks::update(concrete_task)
        .filter(tasks::Column::Id.eq(task_id))
        .exec(&database)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()));
    Ok(())
}

pub async fn partial_task_update(
    Path(task_id): Path<i32>,
    State(database): State<DatabaseConnection>,
    Json(req): Json<TaskRequest>,
) -> Result<(), (StatusCode, String)> {
    let mut task = if let Some(task) = Tasks::find_by_id(task_id)
        .one(&database)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
    {
        task.into_active_model()
    } else {
        return Err((StatusCode::NOT_FOUND, String::new()));
    };
    if let Some(description) = req.description {
        task.description = match description.is_empty() {
            true => Set(None),
            false => Set(Some(description)),
        }
    }
    if let Some(priority) = req.priority {
        task.priority = match priority.is_empty() {
            true => Set(None),
            false => Set(Some(priority)),
        }
    }
    let _ = Tasks::update(task)
        .filter(tasks::Column::Id.eq(task_id))
        .exec(&database)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()));
    Ok(())
}

pub async fn delete_task(
    Path(task_id): Path<i32>,
    State(database): State<DatabaseConnection>,
    Query(query_params): Query<DeleteParams>,
) -> Result<(), (StatusCode, String)> {
    if let Some(soft) = query_params.soft {
        if soft {
            let mut task = if let Some(task) = Tasks::find_by_id(task_id)
                .one(&database)
                .await
                .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
            {
                task.into_active_model()
            } else {
                return Err((StatusCode::NOT_FOUND, String::new()));
            };
            task.deleted_at = Set(Some(chrono::Utc::now().into()));

            let _ = Tasks::update(task)
                .filter(tasks::Column::Id.eq(task_id))
                .exec(&database)
                .await
                .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()));
            return Ok(());
        }
    }
    let _ = Tasks::delete_many()
        .filter(tasks::Column::Id.eq(task_id))
        .exec(&database)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(())
}

fn parse_query_params_into_conditions(params: TaskQueryParams) -> Condition {
    let mut filter = Condition::all();
    filter = filter.add(tasks::Column::DeletedAt.is_null());
    if let Some(title) = params.title {
        filter = if title.is_empty() {
            filter.add(tasks::Column::Title.is_null())
        } else {
            filter.add(tasks::Column::Title.contains(title))
        }
    }
    if let Some(priority) = params.priority {
        filter = if priority.is_empty() {
            filter.add(tasks::Column::Priority.is_null())
        } else {
            filter.add(tasks::Column::Priority.eq(priority))
        }
    }
    filter
}
