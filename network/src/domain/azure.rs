use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AzureLocationsResponse {
    pub subscription_id: String,
    pub locations: Vec<AzureLocation>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AzureLocation {
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub regional_display_name: Option<String>,
    pub region_type: Option<String>,
    pub region_category: Option<String>,
    pub geography_group: Option<String>,
    pub physical_location: Option<String>,
    pub latitude: Option<String>,
    pub longitude: Option<String>,
}
