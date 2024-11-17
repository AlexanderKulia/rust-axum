use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::{
    fs::{exists, File},
    time::Duration,
};

const DB_URL: &str = "sqlite://db.db";

#[tokio::main]
async fn main() {
    if !exists("db.db").unwrap_or(false) {
        File::create("db.db").expect("failed to create database file");
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&DB_URL)
        .await
        .expect("failed to connect to database");

    sqlx::query("create table if not exists users (id integer primary key not null, username varchar(255) not null);").execute(&pool).await.expect("failed to migrate database");

    let app = Router::new()
        .route("/", get(index))
        .route("/users", get(get_users))
        .route("/users", post(create_user))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// routes
async fn index() -> &'static str {
    return "Hello world";
}

async fn get_users(State(pool): State<SqlitePool>) -> (StatusCode, Json<Vec<User>>) {
    let users: Vec<User> = sqlx::query_as("select * from users;")
        .fetch_all(&pool)
        .await
        .expect("failed to load users");
    (StatusCode::OK, Json(users))
}

async fn create_user(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    let user = User {
        id: 1,
        username: payload.username,
    };
    sqlx::query("insert into users (username) values ($1);")
        .bind(&user.username)
        .execute(&pool)
        .await
        .expect("failed to insert user");
    (StatusCode::CREATED, Json(user))
}

#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

#[derive(Serialize, sqlx::FromRow, Debug)]
struct User {
    id: u64,
    username: String,
}
