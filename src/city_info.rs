use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct CityMetaData {
    pub name: String,
    pub id: String,
    pub map: String,
    pub address: String, // "127.0.0.1:8080"
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CityInfo {
    pub simrunning: bool,
    pub elapsed: f64,
    pub population: u32,
}
