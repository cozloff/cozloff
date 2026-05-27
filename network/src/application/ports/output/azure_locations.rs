use async_trait::async_trait;
use crate::domain::azure::AzureLocation;

#[async_trait]
pub trait AzureLocationProvider: Send + Sync {
    async fn list_locations(&self, subscription_id: &str) 
        -> Result<Vec<AzureLocation>, String>;
}