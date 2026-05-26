mod geo;
mod handlers;
mod models;
mod probe;
mod routes;

#[tokio::main]
async fn main() {
    let app = routes::create_router();
    let bind_address =
        std::env::var("API_BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    let listener = tokio::net::TcpListener::bind(&bind_address).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
