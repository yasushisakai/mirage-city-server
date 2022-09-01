mod city_info;
mod handlers;

use axum::Router;
use axum::{
    extract::Extension,
    routing::{get, post, put},
};

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tracing;

use city_info::{CityInfo, CityMetaData};
use handlers::*;

pub struct AppState {
    pub cities: RwLock<HashMap<String, CityMetaData>>, // key = name
    pub city_data: RwLock<HashMap<String, CityInfo>>,  // key = id
}

impl AppState {
    pub fn new() -> Self {
        let cities = RwLock::new(HashMap::new());
        let city_data = RwLock::new(HashMap::new());

        Self { cities, city_data }
    }
}

pub type State = Arc<AppState>;

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState::new());

    tracing_subscriber::fmt::init();

    let city_routes = Router::new()
        .route("/register", post(register))
        .route("/info/:id", put(update).get(info))
        .route("/upload/:id", post(upload))
        .route("/command/:name", post(command));

    let app = Router::new()
        .route("/hello", get(hello))
        .nest("/city", city_routes)
        .route("/cities/list", get(list));

    let api_root = Router::new().nest("/api", app).layer(Extension(state));

    let address = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("starting server");
    axum::Server::bind(&address)
        .serve(api_root.into_make_service())
        .await
        .unwrap();
}
