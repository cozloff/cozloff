use crate::{
    application::ports::input::network_trace::NetworkHopRepository,
    application::ports::output::{NetworkProbe, GeoLocator},
    domain::network::PathsResponse,
};

const DEFAULT_TARGETS: &[&str] = &["1.1.1.1", "8.8.8.8", "9.9.9.9", "208.67.222.222"];

pub struct TracePathsCommand {
    pub targets: Option<String>,
    pub max_hops: Option<u8>,
    pub include_geo: bool,
}

pub struct TracePathsService<P, G, R> {
    probe: P,
    geo_locator: G,
    hop_repository: R,
}

impl<P, G, R> TracePathsService<P, G, R>
where
    P: NetworkProbe,
    G: GeoLocator,
    R: NetworkHopRepository,
{
    pub fn new(probe: P, geo_locator: G, hop_repository: R) -> Self {
        Self {
            probe,
            geo_locator,
            hop_repository,
        }
    }

    pub async fn execute(&self, command: TracePathsCommand) -> Result<PathsResponse, String> {
        let targets = command
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
        let max_hops = command.max_hops.unwrap_or(12).clamp(1, 30);

        let mut traces = Vec::with_capacity(targets.len());
        for target in targets {
            traces.push(self.probe.trace_target(target, max_hops).await?);
        }

        if command.include_geo {
            let locations = self.geo_locator.locate_hops(&traces).await?;
            for trace in &mut traces {
                for hop in &mut trace.hops {
                    if let Some(ip) = &hop.ip {
                        hop.location = locations.get(ip).cloned();
                    }
                }
            }
        }

        self.hop_repository.save_path_traces(&traces).await?;

        Ok(PathsResponse { targets: traces })
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        application::ports::output::{GeoLocator, NetworkHopRepository, NetworkProbe},
        domain::network::{Hop, Location, PathTrace},
    };
    use async_trait::async_trait;
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    #[test]
    fn parse_targets_trims_skips_empty_and_limits_to_eight() {
        let targets = parse_targets(" one.one.one.one, ,8.8.8.8,9.9.9.9,a,b,c,d,e,f ");

        assert_eq!(
            targets,
            vec![
                "one.one.one.one",
                "8.8.8.8",
                "9.9.9.9",
                "a",
                "b",
                "c",
                "d",
                "e"
            ]
        );
    }

    #[tokio::test]
    async fn execute_uses_defaults_and_clamps_max_hops() {
        let probe = RecordingProbe::default();
        let repository = RecordingRepository::default();
        let service = TracePathsService::new(probe.clone(), EmptyGeoLocator, repository.clone());

        let response = service
            .execute(TracePathsCommand {
                targets: None,
                max_hops: Some(99),
                include_geo: false,
            })
            .await
            .unwrap();

        assert_eq!(response.targets.len(), 4);
        assert_eq!(
            probe.calls.lock().unwrap().as_slice(),
            [
                ("1.1.1.1".to_string(), 30),
                ("8.8.8.8".to_string(), 30),
                ("9.9.9.9".to_string(), 30),
                ("208.67.222.222".to_string(), 30)
            ]
        );
        assert_eq!(repository.saved.lock().unwrap().len(), 4);
    }

    #[tokio::test]
    async fn execute_attaches_locations_when_geo_is_enabled() {
        let probe = RecordingProbe::default();
        let repository = RecordingRepository::default();
        let mut locations = HashMap::new();
        locations.insert(
            "8.8.8.8".to_string(),
            Location {
                country: Some("United States".to_string()),
                region_name: None,
                city: None,
                lat: None,
                lon: None,
                isp: None,
                org: None,
            },
        );
        let service = TracePathsService::new(
            probe,
            StaticGeoLocator { locations },
            repository,
        );

        let response = service
            .execute(TracePathsCommand {
                targets: Some("dns.google".to_string()),
                max_hops: Some(0),
                include_geo: true,
            })
            .await
            .unwrap();

        assert_eq!(
            response.targets[0].hops[0].location.as_ref().unwrap().country,
            Some("United States".to_string())
        );
    }

    #[derive(Clone, Default)]
    struct RecordingProbe {
        calls: Arc<Mutex<Vec<(String, u8)>>>,
    }

    #[async_trait]
    impl NetworkProbe for RecordingProbe {
        async fn trace_target(&self, target: String, max_hops: u8) -> Result<PathTrace, String> {
            self.calls
                .lock()
                .unwrap()
                .push((target.clone(), max_hops));
            Ok(PathTrace {
                target,
                hops: vec![Hop {
                    ttl: 1,
                    ip: Some("8.8.8.8".to_string()),
                    rtt_ms: Some(2.5),
                    location: None,
                }],
                error: None,
            })
        }
    }

    struct EmptyGeoLocator;

    #[async_trait]
    impl GeoLocator for EmptyGeoLocator {
        async fn locate_hops(
            &self,
            _traces: &[PathTrace],
        ) -> Result<HashMap<String, Location>, String> {
            Ok(HashMap::new())
        }
    }

    struct StaticGeoLocator {
        locations: HashMap<String, Location>,
    }

    #[async_trait]
    impl GeoLocator for StaticGeoLocator {
        async fn locate_hops(
            &self,
            _traces: &[PathTrace],
        ) -> Result<HashMap<String, Location>, String> {
            Ok(self.locations.clone())
        }
    }

    #[derive(Clone, Default)]
    struct RecordingRepository {
        saved: Arc<Mutex<Vec<PathTrace>>>,
    }

    #[async_trait]
    impl NetworkHopRepository for RecordingRepository {
        async fn save_path_traces(&self, traces: &[PathTrace]) -> Result<(), String> {
            self.saved.lock().unwrap().extend_from_slice(traces);
            Ok(())
        }
    }
}
