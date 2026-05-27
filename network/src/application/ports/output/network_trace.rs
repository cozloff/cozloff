use async_trait::async_trait;
use std::collections::HashMap;
use crate::domain::network::{Location, PathTrace};

#[async_trait]
pub trait NetworkProbe: Send + Sync {
    async fn trace_target(&self, target: String, max_hops: u8)
        -> Result<PathTrace, String>;
}

#[async_trait]
pub trait GeoLocator: Send + Sync {
    async fn locate_hops(&self, traces: &[PathTrace])
        -> Result<HashMap<String, Location>, String>;
}