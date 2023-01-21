mod data;
mod kanji;
use std::{net::SocketAddr, sync::Arc};

use axum::{http::StatusCode, response::IntoResponse, routing::get, Extension, Router};
use std::env;
use tower_http::trace::TraceLayer;

pub struct Config {
    redis_url: String,
    mongo_url: String,
    server_port: u16,
}

pub enum AppError {
    Error(String),
    // RedisError(RedisError),
    MongoError(mongodb::error::Error),
    SerdeError(serde_json::Error),
}

fn get_config() -> Config {
    Config {
        redis_url: env::var("REDIS_URL").unwrap(),
        mongo_url: env::var("MONGODB_URL").unwrap(),
        server_port: env::var("SERVER_PORT").unwrap().parse().unwrap(),
    }
}

type Database = Arc<mongodb::Database>;

#[tokio::main]
async fn main() {
    let config = get_config();
    // let client = redis::Client::open(config.redis_url).unwrap();
    // let state = Arc::new(client);

    let db = mongodb::Client::with_uri_str(config.mongo_url)
        .await
        .unwrap()
        .database("kanjisho");
    let state = Arc::new(db);

    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(|| async { "pong" }))
        .route("/kanjidic", get(kanji::get_index))
        .route("/kanjidic/random", get(kanji::get_random))
        .route("/kanjidic/dict", get(kanji::get_dict_entries))
        .route("/kanjidic/dict/:dict/:entry", get(kanji::get_dict_entry))
        .route("/kanjidic/search", get(kanji::get_search))
        .route("/kanjidic/:kanji", get(kanji::get_kanji))
        .layer(Extension(state))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// impl From<RedisError> for AppError {
//     fn from(e: RedisError) -> Self {
//         AppError::RedisError(e)
//     }
// }

impl From<mongodb::error::Error> for AppError {
    fn from(e: mongodb::error::Error) -> Self {
        AppError::MongoError(e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::SerdeError(e)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let body = match self {
            AppError::Error(e) => e,
            // AppError::RedisError(e) => e.to_string(),
            AppError::MongoError(e) => e.to_string(),
            AppError::SerdeError(e) => e.to_string(),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
