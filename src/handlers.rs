use crate::city_info::CityInfo;
use crate::{city_info::CityMetaData, State};
use axum::body::Bytes;
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;
use axum::{extract, response::IntoResponse, Extension};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io;
use tokio::fs::File;
use tokio::{self, io::AsyncWriteExt, net::TcpStream};
use tracing::debug;

pub async fn hello(
    Extension(state): Extension<State>,
    extract::Path(name): extract::Path<String>,
) -> impl IntoResponse {
    // TODO
    let address = get_address_from_name(state, &name).unwrap();
    send_command(address, "hello").await.unwrap()
}

pub async fn list(Extension(state): Extension<State>) -> impl IntoResponse {
    let cities = state.cities.read().unwrap();
    Json(
        cities
            .iter()
            .map(|(_, v)| v.to_owned())
            .collect::<Vec<CityMetaData>>(),
    )
}

pub async fn register(
    Extension(state): Extension<State>,
    extract::Json(payload): extract::Json<CityMetaData>,
) -> impl IntoResponse {
    debug!("register");
    let mut cities = state.cities.write().unwrap();
    cities.insert(payload.name.to_string(), payload);
}

pub async fn update(
    Extension(state): Extension<State>,
    extract::Json(payload): extract::Json<CityInfo>,
    extract::Path(id): extract::Path<String>,
) -> impl IntoResponse {
    debug!("update");
    let mut info = state.city_data.write().unwrap();
    info.insert(id, payload);
    "OK"
}

pub async fn info(
    Extension(state): Extension<State>,
    extract::Path(name): extract::Path<String>,
) -> Response {
    debug!("info");
    let cities = state.cities.read().unwrap();

    if !cities.contains_key(&name) {
        return (StatusCode::BAD_REQUEST, "city not found").into_response();
    }

    let meta = cities.get(&name).unwrap();

    let info = state.city_data.read().unwrap();

    if !info.contains_key(&meta.id) {
        return (
            StatusCode::BAD_REQUEST,
            "city was found, but no info. (yet)",
        )
            .into_response();
    }

    let city = info.get(&meta.id).unwrap();

    (StatusCode::OK, Json(city.clone())).into_response()
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Command {
    name: String,
}

pub async fn command(
    Extension(state): Extension<State>,
    extract::Path(name): extract::Path<String>,
    extract::Json(command): extract::Json<Command>,
) -> Response {
    let address = get_address_from_name(state, &name).unwrap();

    debug!("cmd: {}", command.name);

    send_command(address, command.name)
        .await
        .unwrap()
        .into_response()
}

fn get_address_from_name(state: State, name: &str) -> Option<String> {
    let cities = state.cities.read().unwrap();

    if !cities.contains_key(name) {
        return None;
    }

    let meta = cities.get(name).unwrap();

    Some(meta.address.to_string())
}

pub async fn upload(bytes: Bytes) -> impl IntoResponse {
    debug!("upload");
    debug!("{}", bytes.len());

    let path = format!("/home/ec2-user/html/images/screenshot.png");
    debug!("saving file as {path}");
    match save_png(path, bytes).await {
        Ok(()) => (StatusCode::OK, "OK"),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed to save image"),
    }
}

async fn save_png(file_name: impl AsRef<str>, bytes: Bytes) -> std::io::Result<()> {
    let mut buffer = File::create(file_name.as_ref()).await?;
    buffer.write_all(&bytes).await?;
    Ok(())
}

async fn send_command(
    address: impl AsRef<str>,
    s: impl AsRef<str>,
) -> Result<String, Box<dyn Error>> {
    let mut stream = TcpStream::connect(address.as_ref()).await?;
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
