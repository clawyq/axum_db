mod database;
mod routes;
mod utils;

use axum_db::connect_to_db;
use routes::create_routes;
use std::{env, fmt};
use tracing_subscriber;

#[derive(PartialEq)]
enum AppEnv {
    Dev,
    Prod,
}

impl fmt::Display for AppEnv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppEnv::Dev => write!(f, "dev"),
            AppEnv::Prod => write!(f, "prod"),
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let db_conn_str = get_db_conn_str();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let connection = match connect_to_db(&db_conn_str[..]).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect to database: {e}");
            return;
        }
    };
    axum::serve(listener, create_routes(connection).await)
        .await
        .unwrap();
}

fn get_db_conn_str() -> String {
    let app_env = match env::var("APP_ENV") {
        Ok(v) if v == "prod" => AppEnv::Prod,
        _ => AppEnv::Dev,
    };

    println!("Running in {app_env} mode");

    if app_env == AppEnv::Dev {
        match dotenvy::dotenv() {
            Ok(path) => println!(".env read successfully from {}", path.display()),
            Err(e) => println!("Could not load .env file: {e}"),
        };
    }
    env::var("DATABASE_URL").expect("DB connection string not set")
}
