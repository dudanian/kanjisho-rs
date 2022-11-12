use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::Path, http::StatusCode, response::IntoResponse, routing::get, Extension, Router,
};
use redis::{Commands, ConnectionLike, RedisError};
use std::env;

struct Config {
    redis_url: String,
    server_port: u16,
}

enum AppError {
    RedisError(RedisError),
}

fn get_config() -> Config {
    Config {
        redis_url: env::var("REDIS_URL").unwrap(),
        server_port: env::var("SERVER_PORT").unwrap().parse().unwrap(),
    }
}

#[tokio::main]
async fn main() {
    let config = get_config();
    let client = redis::Client::open(config.redis_url).unwrap();
    let state = Arc::new(client);

    let app = Router::new()
        .route("/", get(|| async { "pong" }))
        .route("/kanjidic", get(get_kanjidic_index))
        .route("/kanjidic/:kanji", get(get_kanjidic))
        .layer(Extension(state));

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_kanjidic_index(client: Extension<Arc<redis::Client>>) -> Result<String, AppError> {
    let mut con = client.get_connection()?;
    Ok(con.get("index")?)
}

async fn get_kanjidic(
    Path(kanji): Path<String>,
    client: Extension<Arc<redis::Client>>,
) -> Result<String, AppError> {
    let mut con = client.get_connection()?;
    Ok(con.get(kanji)?)
}

impl From<RedisError> for AppError {
    fn from(e: RedisError) -> Self {
        AppError::RedisError(e)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let body = match self {
            AppError::RedisError(e) => e.to_string(),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
