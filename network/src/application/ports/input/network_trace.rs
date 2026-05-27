use async_trait::async_trait;
use std::collections::HashMap;
use crate::domain::network::PathTrace;

#[async_trait]
pub trait NetworkHopRepository: Send + Sync {
    async fn save_path_traces(&self, traces: &[PathTrace]) 
        -> Result<(), String>;
}