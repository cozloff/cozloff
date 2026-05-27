mod application;
mod domain;
mod infrastructure;

use infrastructure::adapters::{
    input::{handlers::AppState, routes},
    output::{
        azure::AzureResourceManagerLocationProvider, questdb::QuestDbNetworkHopRepository,
    },
};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Bind on 8080 locally
    let bind_address = std::env::var("API_BIND_ADDRESS")
            .unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    // Quest DB connection
    let database_url = std::env::var("QUESTDB_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://admin:quest@127.0.0.1:8812/qdb".to_string());
    let hop_repository = QuestDbNetworkHopRepository::connect(&database_url)
        .await
        .expect("failed to connect to QuestDB");

    // Azure location provider
    let azure_location_provider =
        AzureResourceManagerLocationProvider::new().expect("failed to create Azure client");
    let state = Arc::new(AppState {
        hop_repository,
        azure_location_provider,
    });

    // Create the router and serve the application
    let app = routes::create_router(state);
    let listener = tokio::net::TcpListener::bind(&bind_address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
