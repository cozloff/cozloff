use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct PathsResponse {
    pub targets: Vec<PathTrace>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PathTrace {
    pub target: String,
    pub hops: Vec<Hop>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Hop {
    pub ttl: u8,
    pub ip: Option<String>,
    pub rtt_ms: Option<f32>,
    pub location: Option<Location>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Location {
    pub country: Option<String>,
    pub region_name: Option<String>,
    pub city: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub isp: Option<String>,
    pub org: Option<String>,
}
