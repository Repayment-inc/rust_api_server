// 参考サイト https://medium.com/intelliconnect-engineering/using-axum-framework-to-create-rest-api-part-1-7d434d2c5de4

use axum::{
    routing::{get, post},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};

use once_cell::sync::Lazy;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde::{Deserialize, Serialize};

mod controllers;
mod error;
mod models;
mod utils;

// secret key for JWT token
static KEYS: Lazy<models::auth::Keys> = Lazy::new(|| {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "Your secret here".to_owned());
    models::auth::Keys::new(secret.as_bytes())
});

// コンパイル時、環境変数設定が必要
// DATABASE_URL=postgresql://ユーザ名:パスワード@localhost:5432/mydb cargo run

#[tokio::main]
async fn main() {
    let durl = std::env::var("DATABASE_URL").expect("set DATABASE_URL env variable");

    // initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "axum_api=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cors = CorsLayer::new().allow_origin(Any);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&durl)
        .await
        .expect("unable to connect to database");

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/register", post(controllers::auth::register))
        .route("/login", post(controllers::auth::login))
        //only loggedin user can access this route
        .route("/user_profile", get(controllers::user::user_profile))
        .layer(cors)
        .with_state(pool);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("failed to start server");
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

