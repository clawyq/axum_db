use sea_orm::Database;

pub async fn run(db_conn_str: &str) {
    let db = Database::connect(db_conn_str).await; 
}
