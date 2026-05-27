use crate::{
    application::ports::output::azure_locations::AzureLocationProvider, 
    domain::azure::AzureLocation
};
use async_trait::async_trait;
use azure_mgmt_resources::package_subscriptions_2021_01::{
    Client,
    models::{Location, location_metadata},
};

#[derive(Clone)]
pub struct AzureResourceManagerLocationProvider {
    client: Client,
}

impl AzureResourceManagerLocationProvider {
    pub fn new() -> Result<Self, String> {
        let credential = azure_identity::create_credential()
            .map_err(|error| format!("failed to create Azure credential: {error}"))?;
        let client = Client::builder(credential)
            .build()
            .map_err(|error| format!("failed to create Azure Resource Manager client: {error}"))?;

        Ok(Self { client })
    }
}

#[async_trait]
impl AzureLocationProvider for AzureResourceManagerLocationProvider {
    async fn list_locations(&self, subscription_id: &str) -> Result<Vec<AzureLocation>, String> {
        let response = self
            .client
            .subscriptions_client()
            .list_locations(subscription_id)
            .include_extended_locations(true)
            .send()
            .await
            .map_err(|error| format!("failed to list Azure locations: {error}"))?
            .into_body()
            .await
            .map_err(|error| format!("failed to parse Azure locations: {error}"))?;

        Ok(response.value.into_iter().map(map_location).collect())
    }
}

fn map_location(location: Location) -> AzureLocation {
    let metadata = location.metadata;

    AzureLocation {
        name: location.name,
        display_name: location.display_name,
        regional_display_name: location.regional_display_name,
        region_type: metadata
            .as_ref()
            .and_then(|metadata| metadata.region_type.as_ref())
            .map(map_region_type),
        region_category: metadata
            .as_ref()
            .and_then(|metadata| metadata.region_category.as_ref())
            .map(map_region_category),
        geography_group: metadata
            .as_ref()
            .and_then(|metadata| metadata.geography_group.clone()),
        physical_location: metadata
            .as_ref()
            .and_then(|metadata| metadata.physical_location.clone()),
        latitude: metadata.as_ref().and_then(|metadata| metadata.latitude.clone()),
        longitude: metadata.and_then(|metadata| metadata.longitude),
    }
}

fn map_region_type(region_type: &location_metadata::RegionType) -> String {
    match region_type {
        location_metadata::RegionType::Physical => "Physical",
        location_metadata::RegionType::Logical => "Logical",
        location_metadata::RegionType::UnknownValue(value) => value,
    }
    .to_string()
}

fn map_region_category(region_category: &location_metadata::RegionCategory) -> String {
    match region_category {
        location_metadata::RegionCategory::Recommended => "Recommended",
        location_metadata::RegionCategory::Extended => "Extended",
        location_metadata::RegionCategory::Other => "Other",
        location_metadata::RegionCategory::UnknownValue(value) => value,
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use azure_mgmt_resources::package_subscriptions_2021_01::models::LocationMetadata;

    #[test]
    fn map_location_flattens_sdk_location_metadata() {
        let location = Location {
            name: Some("westus".to_string()),
            display_name: Some("West US".to_string()),
            regional_display_name: Some("(US) West US".to_string()),
            metadata: Some(LocationMetadata {
                region_type: Some(location_metadata::RegionType::Physical),
                region_category: Some(location_metadata::RegionCategory::Recommended),
                geography_group: Some("US".to_string()),
                physical_location: Some("California".to_string()),
                latitude: Some("37.783".to_string()),
                longitude: Some("-122.417".to_string()),
                ..LocationMetadata::default()
            }),
            ..Location::default()
        };

        let mapped = map_location(location);

        assert_eq!(mapped.name, Some("westus".to_string()));
        assert_eq!(mapped.region_type, Some("Physical".to_string()));
        assert_eq!(mapped.region_category, Some("Recommended".to_string()));
        assert_eq!(mapped.physical_location, Some("California".to_string()));
    }
}
