use axum::{
    body::Bytes,
    extract::{self, Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use redis_async::client::{paired::PairedConnection, paired_connect};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::{error::Error, io};
use tokio::{self, io::AsyncWriteExt, net::TcpStream, spawn};
// use tower_http::cors::CorsLayer;

struct AppState {
    redis: PairedConnection,
    name_id: RwLock<HashMap<String, String>>,
    id_address: RwLock<HashMap<String, String>>,
}

impl AppState {
    pub fn new(redis: PairedConnection) -> Self {
        let name_id = RwLock::new(HashMap::new());
        let id_address = RwLock::new(HashMap::new());

        Self {
            redis,
            name_id,
            id_address,
        }
    }
}

type State = Arc<AppState>;

#[derive(Deserialize, Debug, Clone)]
struct RegisterCity {
    address: String,
    name: String,
    id: String,
}

#[tokio::main]
async fn main() {
    let redis = paired_connect("127.0.0.1:6379").await.unwrap();

    let state = Arc::new(AppState::new(redis));

    let app = Router::new()
        .route("/", get(hello))
        .route("/shoot", get(shoot))
        .route("/latest_ss", get(latest_screenshot))
        .route("/register", post(register))
        .route("/city/meta/:city_name", get(city_meta))
        .route("/city/:city_name/upload_image", post(upload_image))
        .layer(Extension(state));

    let address = SocketAddr::from(([127, 0, 0, 1], 3000));

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn hello() -> impl IntoResponse {
    send_command("toggle sim").await.unwrap()
}

async fn shoot() -> impl IntoResponse {
    send_command("shoot").await.unwrap()
}

async fn latest_screenshot() -> impl IntoResponse {
    send_command("latest ss").await.unwrap()
}

async fn upload_image(body: String) -> impl IntoResponse {
    let mut file = File::create("based.jpg").unwrap();
    let decoded = base64::decode(body).unwrap();
    file.write_all(&decoded).unwrap();
    // write!(file, "{}", &body).unwrap();
    "OK"
}

async fn register(
    Extension(state): Extension<State>,
    extract::Json(payload): extract::Json<RegisterCity>,
) -> impl IntoResponse {
    let mut name_id = state.name_id.write().unwrap();

    if name_id.contains_key(&payload.name) {
        return (StatusCode::BAD_REQUEST, "city name is already taken");
    }

    let mut id_address = state.id_address.write().unwrap();

    name_id.insert(payload.name, payload.id.clone());
    id_address.insert(payload.id, payload.address);

    (StatusCode::OK, "OK")
}

async fn city_meta(
    Extension(state): Extension<State>,
    Path(city_name): Path<String>,
) -> impl IntoResponse {
    let name_id = state.name_id.read().unwrap();

    if !name_id.contains_key(&city_name) {
        return (StatusCode::BAD_REQUEST, "city name not found".to_string());
    }

    let id = name_id.get(&city_name).unwrap();

    let id_address = state.id_address.read().unwrap();

    let address = id_address.get(id).unwrap();

    (StatusCode::OK, address.to_string())
}

async fn send_command(s: impl AsRef<str>) -> Result<String, Box<dyn Error>> {
    let mut stream = TcpStream::connect("18.27.123.81:9000").await?;
    stream.write_all(&s.as_ref().as_bytes()).await?;

    loop {
        stream.readable().await?;

        let mut buf = [0; 1024];

        match stream.try_read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let mes = String::from_utf8_lossy(&buf[..n]);
                return Ok(format!("response: {mes}"));
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok("OK".to_string())
}
