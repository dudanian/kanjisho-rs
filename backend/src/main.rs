use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::Path, http::StatusCode, response::IntoResponse, routing::get, Extension, Json, Router,
};
use mongodb::bson::doc;
use parse::kanjidic::{self, Kanji};
use std::env;
use tower_http::trace::TraceLayer;
mod db;

type Database = mongodb::Database;

pub struct Config {
    redis_url: String,
    mongo_url: String,
    server_port: u16,
}

enum AppError {
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
        .route("/kanjidic", get(get_kanjidic_index))
        .route("/kanjidic/:kanji", get(get_kanjidic))
        .layer(Extension(state))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_kanjidic_index(db: Extension<Arc<Database>>) -> Result<Json<Vec<String>>, AppError> {

    let out = db.collection::<Kanji>("kanjidic").distinct("literal", None, None).await?;
    Ok(Json(out.iter().map(|b| b.to_string()).collect()))
}

async fn get_kanjidic(
    Path(kanji): Path<String>,
    db: Extension<Arc<Database>>,
) -> Result<Json<kanjidic::Kanji>, AppError> {
    // let mut con = client.get_connection()?;
    // let raw: String = con.get(kanji)?;
    // let kanji: kanjidic::Kanji = serde_json::from_str(&raw)?;

    let out = db
        .collection::<Kanji>("kanjidic")
        .find_one(doc! { "literal": kanji}, None)
        .await?;

    Ok(Json(out.unwrap()))
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
            // AppError::RedisError(e) => e.to_string(),
            AppError::MongoError(e) => e.to_string(),
            AppError::SerdeError(e) => e.to_string(),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
