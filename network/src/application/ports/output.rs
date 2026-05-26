use crate::domain::{
    azure::AzureLocation,
    network::{Location, PathTrace},
};
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait NetworkProbe: Send + Sync {
    async fn trace_target(&self, target: String, max_hops: u8) -> Result<PathTrace, String>;
}

#[async_trait]
pub trait GeoLocator: Send + Sync {
    async fn locate_hops(&self, traces: &[PathTrace]) -> Result<HashMap<String, Location>, String>;
}

#[async_trait]
pub trait NetworkHopRepository: Send + Sync {
    async fn save_path_traces(&self, traces: &[PathTrace]) -> Result<(), String>;
}

#[async_trait]
pub trait AzureLocationProvider: Send + Sync {
    async fn list_locations(&self, subscription_id: &str) -> Result<Vec<AzureLocation>, String>;
}
