use crate::{geo, models::PathsResponse, probe};
use axum::{Json, extract::Query};
use serde::Deserialize;

const DEFAULT_TARGETS: &[&str] = &["1.1.1.1", "8.8.8.8", "9.9.9.9", "208.67.222.222"];

#[derive(Debug, Deserialize)]
pub struct PathsQuery {
    targets: Option<String>,
    max_hops: Option<u8>,
}

pub async fn network_paths(Query(query): Query<PathsQuery>) -> Json<PathsResponse> {
    let targets = query
        .targets
        .as_deref()
        .map(parse_targets)
        .filter(|targets| !targets.is_empty())
        .unwrap_or_else(|| {
            DEFAULT_TARGETS
                .iter()
                .map(|target| target.to_string())
                .collect()
        });
    let max_hops = query.max_hops.unwrap_or(12).clamp(1, 30);

    let mut traces = Vec::with_capacity(targets.len());
    for target in targets {
        traces.push(probe::trace_target(target, max_hops).await);
    }

    let locations = geo::locate_hops(&traces).await;
    for trace in &mut traces {
        for hop in &mut trace.hops {
            if let Some(ip) = &hop.ip {
                hop.location = locations.get(ip).cloned();
            }
        }
    }

    Json(PathsResponse { targets: traces })
}

pub async fn network_paths_no_geo(Query(query): Query<PathsQuery>) -> Json<PathsResponse> {
    let targets = query
        .targets
        .as_deref()
        .map(parse_targets)
        .filter(|targets| !targets.is_empty())
        .unwrap_or_else(|| {
            DEFAULT_TARGETS
                .iter()
                .map(|target| target.to_string())
                .collect()
        });
    let max_hops = query.max_hops.unwrap_or(12).clamp(1, 30);

    let mut traces = Vec::with_capacity(targets.len());
    for target in targets {
        traces.push(probe::trace_target(target, max_hops).await);
    }

    Json(PathsResponse { targets: traces })
}

fn parse_targets(targets: &str) -> Vec<String> {
    targets
        .split(',')
        .map(str::trim)
        .filter(|target| !target.is_empty())
        .take(8)
        .map(ToOwned::to_owned)
        .collect()
}
