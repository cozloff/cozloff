use crate::{
    application::services::{
        azure_locations::{ListAzureLocationsCommand, ListAzureLocationsService},
        trace_paths::{TracePathsCommand, TracePathsService},
    },
    domain::{azure::AzureLocationsResponse, network::PathsResponse},
    infrastructure::adapters::output::{
        azure::AzureResourceManagerLocationProvider, geo::IpApiGeoLocator,
        probe::CommandNetworkProbe, questdb::QuestDbNetworkHopRepository,
    },
};
use axum::{Json, extract::Query, extract::State};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub hop_repository: QuestDbNetworkHopRepository,
    pub azure_location_provider: AzureResourceManagerLocationProvider,
}

#[derive(Debug, Deserialize)]
pub struct PathsQuery {
    targets: Option<String>,
    max_hops: Option<u8>,
}

pub async fn network_openapi() -> Json<Value> {
    Json(json!({
        "openapi": "3.1.0",
        "info": {
            "title": "Network API",
            "version": "0.1.0",
            "description": "Trace network paths to one or more targets."
        },
        "paths": {
            "/network/azure/locations": {
                "get": {
                    "summary": "List Azure datacenter locations",
                    "description": "Lists Azure Resource Manager locations for the subscription configured by AZURE_SUBSCRIPTION_ID.",
                    "responses": {
                        "200": {
                            "description": "Azure locations for the configured subscription.",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/AzureLocationsResponse" }
                                }
                            }
                        }
                    }
                }
            },
            "/network/paths": {
                "get": path_operation(
                    "Trace network paths",
                    "Runs path tracing and includes IP geolocation for discovered hops.",
                )
            },
            "/network/paths/no-geo": {
                "get": path_operation(
                    "Trace network paths without geolocation",
                    "Runs path tracing without geolocation lookups.",
                )
            }
        },
        "components": {
            "schemas": {
                "AzureLocationsResponse": {
                    "type": "object",
                    "required": ["subscription_id", "locations"],
                    "properties": {
                        "subscription_id": { "type": "string" },
                        "locations": {
                            "type": "array",
                            "items": { "$ref": "#/components/schemas/AzureLocation" }
                        },
                        "error": { "type": ["string", "null"] }
                    }
                },
                "AzureLocation": {
                    "type": "object",
                    "properties": {
                        "name": { "type": ["string", "null"] },
                        "display_name": { "type": ["string", "null"] },
                        "regional_display_name": { "type": ["string", "null"] },
                        "region_type": { "type": ["string", "null"] },
                        "region_category": { "type": ["string", "null"] },
                        "geography_group": { "type": ["string", "null"] },
                        "physical_location": { "type": ["string", "null"] },
                        "latitude": { "type": ["string", "null"] },
                        "longitude": { "type": ["string", "null"] }
                    }
                },
                "PathsResponse": {
                    "type": "object",
                    "required": ["targets"],
                    "properties": {
                        "targets": {
                            "type": "array",
                            "items": { "$ref": "#/components/schemas/PathTrace" }
                        }
                    }
                },
                "PathTrace": {
                    "type": "object",
                    "required": ["target", "hops"],
                    "properties": {
                        "target": { "type": "string" },
                        "hops": {
                            "type": "array",
                            "items": { "$ref": "#/components/schemas/Hop" }
                        },
                        "error": { "type": ["string", "null"] }
                    }
                },
                "Hop": {
                    "type": "object",
                    "required": ["ttl"],
                    "properties": {
                        "ttl": { "type": "integer", "format": "uint8", "minimum": 0, "maximum": 255 },
                        "ip": { "type": ["string", "null"] },
                        "rtt_ms": { "type": ["number", "null"], "format": "float" },
                        "location": {
                            "oneOf": [
                                { "$ref": "#/components/schemas/Location" },
                                { "type": "null" }
                            ]
                        }
                    }
                },
                "Location": {
                    "type": "object",
                    "properties": {
                        "country": { "type": ["string", "null"] },
                        "region_name": { "type": ["string", "null"] },
                        "city": { "type": ["string", "null"] },
                        "lat": { "type": ["number", "null"], "format": "double" },
                        "lon": { "type": ["number", "null"], "format": "double" },
                        "isp": { "type": ["string", "null"] },
                        "org": { "type": ["string", "null"] }
                    }
                }
            }
        }
    }))
}

pub async fn network_paths(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PathsQuery>,
) -> Json<PathsResponse> {
    trace_paths(state, query, true).await
}

pub async fn network_paths_no_geo(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PathsQuery>,
) -> Json<PathsResponse> {
    trace_paths(state, query, false).await
}

pub async fn azure_locations(
    State(state): State<Arc<AppState>>,
) -> Json<AzureLocationsResponse> {
    let service = ListAzureLocationsService::new(state.azure_location_provider.clone());
    let subscription_id = std::env::var("AZURE_SUBSCRIPTION_ID").unwrap_or_default();

    match service
        .execute(ListAzureLocationsCommand { subscription_id })
        .await
    {
        Ok(response) => Json(response),
        Err(error) => Json(AzureLocationsResponse {
            subscription_id: String::new(),
            locations: Vec::new(),
            error: Some(error),
        }),
    }
}

async fn trace_paths(
    state: Arc<AppState>,
    query: PathsQuery,
    include_geo: bool,
) -> Json<PathsResponse> {
    let service = TracePathsService::new(
        CommandNetworkProbe,
        IpApiGeoLocator,
        state.hop_repository.clone(),
    );
    let command = TracePathsCommand {
        targets: query.targets,
        max_hops: query.max_hops,
        include_geo,
    };

    match service.execute(command).await {
        Ok(response) => Json(response),
        Err(error) => Json(PathsResponse {
            targets: vec![crate::domain::network::PathTrace {
                target: "request".to_string(),
                hops: Vec::new(),
                error: Some(error),
            }],
        }),
    }
}

fn path_operation(summary: &str, description: &str) -> Value {
    json!({
        "summary": summary,
        "description": description,
        "parameters": [
            {
                "name": "targets",
                "in": "query",
                "required": false,
                "description": "Comma-separated hostnames or IP addresses to trace.",
                "schema": { "type": "string" },
                "example": "example.com,1.1.1.1"
            },
            {
                "name": "max_hops",
                "in": "query",
                "required": false,
                "description": "Maximum number of hops to probe.",
                "schema": { "type": "integer", "format": "uint8", "minimum": 1, "maximum": 255 },
                "example": 30
            }
        ],
        "responses": {
            "200": {
                "description": "Network path traces.",
                "content": {
                    "application/json": {
                        "schema": { "$ref": "#/components/schemas/PathsResponse" }
                    }
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn network_openapi_documents_existing_network_routes() {
        let Json(openapi) = network_openapi().await;

        assert_eq!(openapi["openapi"], "3.1.0");
        assert_eq!(openapi["info"]["title"], "Network API");
        assert!(openapi["paths"]["/network/azure/locations"].is_object());
        assert!(openapi["paths"]["/network/paths"].is_object());
        assert!(openapi["paths"]["/network/paths/no-geo"].is_object());
        assert_eq!(
            openapi["components"]["schemas"]["PathsResponse"]["properties"]["targets"]["items"]
                ["$ref"],
            "#/components/schemas/PathTrace"
        );
    }

    #[test]
    fn path_operation_includes_supported_query_parameters_and_response_schema() {
        let operation = path_operation("Summary", "Description");

        assert_eq!(operation["summary"], "Summary");
        assert_eq!(operation["description"], "Description");
        assert_eq!(operation["parameters"][0]["name"], "targets");
        assert_eq!(operation["parameters"][1]["name"], "max_hops");
        assert_eq!(
            operation["responses"]["200"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/PathsResponse"
        );
    }
}
