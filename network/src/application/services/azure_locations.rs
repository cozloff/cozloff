use crate::{
    application::ports::output::azure_locations::AzureLocationProvider,
    domain::azure::AzureLocationsResponse,
};

pub struct ListAzureLocationsCommand {
    pub subscription_id: String,
}

pub struct ListAzureLocationsService<P> {
    location_provider: P,
}

impl<P> ListAzureLocationsService<P>
where
    P: AzureLocationProvider,
{
    pub fn new(location_provider: P) -> Self {
        Self { location_provider }
    }

    pub async fn execute(
        &self,
        command: ListAzureLocationsCommand,
    ) -> Result<AzureLocationsResponse, String> {
        let subscription_id = command.subscription_id.trim();
        if subscription_id.is_empty() {
            return Err("AZURE_SUBSCRIPTION_ID is required".to_string());
        }

        let locations = self.location_provider.list_locations(subscription_id).await?;

        Ok(AzureLocationsResponse {
            subscription_id: subscription_id.to_string(),
            locations,
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::azure::AzureLocation;
    use async_trait::async_trait;

    #[tokio::test]
    async fn execute_rejects_empty_subscription_id() {
        let service = ListAzureLocationsService::new(StaticLocationProvider);

        let error = service
            .execute(ListAzureLocationsCommand {
                subscription_id: " ".to_string(),
            })
            .await
            .unwrap_err();

        assert_eq!(error, "AZURE_SUBSCRIPTION_ID is required");
    }

    #[tokio::test]
    async fn execute_returns_locations_for_subscription() {
        let service = ListAzureLocationsService::new(StaticLocationProvider);

        let response = service
            .execute(ListAzureLocationsCommand {
                subscription_id: "sub-123".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(response.subscription_id, "sub-123");
        assert_eq!(response.locations[0].name, Some("westus".to_string()));
        assert_eq!(response.error, None);
    }

    struct StaticLocationProvider;

    #[async_trait]
    impl AzureLocationProvider for StaticLocationProvider {
        async fn list_locations(
            &self,
            _subscription_id: &str,
        ) -> Result<Vec<AzureLocation>, String> {
            Ok(vec![AzureLocation {
                name: Some("westus".to_string()),
                display_name: Some("West US".to_string()),
                regional_display_name: None,
                region_type: None,
                region_category: None,
                geography_group: None,
                physical_location: None,
                latitude: None,
                longitude: None,
            }])
        }
    }
}
