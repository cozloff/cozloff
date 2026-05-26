use crate::infrastructure::adapters::input::handlers::{self, AppState};
use axum::{Router, routing::get};
use serde_json::json;
use std::sync::Arc;

pub fn create_router(state: Arc<AppState>) -> Router {
    let scalar_configuration = json!({
        "url": "/network/openapi.json",
        "agent": { "disabled": true },
    });

    let api_routes = Router::new()
        .route("/network/azure/locations", get(handlers::azure_locations))
        .route("/network/paths", get(handlers::network_paths))
        .route("/network/paths/no-geo", get(handlers::network_paths_no_geo))
        .with_state(state);

    Router::new()
        .route(
            "/network",
            get(move || {
                let scalar_configuration = scalar_configuration.clone();
                async move {
                    scalar_api_reference::axum::scalar_response(&scalar_configuration, None)
                }
            }),
        )
        .route("/network/openapi.json", get(handlers::network_openapi))
        .merge(api_routes)
}
