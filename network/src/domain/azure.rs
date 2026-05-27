use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AzureLocationsResponse {
    pub subscription_id: String,        // The Azure subscription ID for which the locations were requested
    pub locations: Vec<AzureLocation>,  // A list of Azure locations available under the specified subscription
    pub error: Option<String>,          // An optional error message if the request to fetch locations failed
}

#[derive(Debug, Clone, Serialize)]
pub struct AzureLocation {
    pub name: Option<String>,                   // The unique name of the Azure location (e.g., "eastus", "westeurope")
    pub display_name: Option<String>,           // A human-readable name for the Azure location (e.g., "East US", "West Europe")
    pub regional_display_name: Option<String>,  // Display name with region info (e.g., "East US (Virginia)", "West Europe (Netherlands)")
    pub region_type: Option<String>,            // The type of the Azure region (e.g., "Physical", "Virtual", "EdgeZone")
    pub region_category: Option<String>,        // The category of the Azure region (e.g., "Recommended", "Other", "Unavailable")
    pub geography_group: Option<String>,        // The geography group of the Azure location (e.g., "US", "EU", "APAC")
    pub physical_location: Option<String>,      // The physical location of the Azure data center (e.g., "Virginia", "Netherlands")
    pub latitude: Option<String>,               // The latitude of the Azure location
    pub longitude: Option<String>,              // The longitude of the Azure location
}
