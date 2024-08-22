use std::io::Error;

use sea_orm::{Database, DatabaseConnection};

pub async fn run(db_conn_str: &str) -> Result<DatabaseConnection, sea_orm::DbErr> {
    Database::connect(db_conn_str).await
}
