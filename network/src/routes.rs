use crate::handlers;
use axum::{Router, routing::get};

pub fn create_router() -> Router {
    Router::new()
        .route("/network/paths", get(handlers::network_paths))
        .route("/network/paths/no-geo", get(handlers::network_paths_no_geo))
}
