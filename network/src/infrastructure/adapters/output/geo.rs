use crate::{
    application::ports::output::GeoLocator,
    domain::network::{Location, PathTrace},
};
use async_trait::async_trait;
use serde::Deserialize;
use std::{collections::HashMap, net::IpAddr, time::Duration};

#[derive(Clone, Default)]
pub struct IpApiGeoLocator;

#[derive(Debug, Deserialize)]
struct IpApiResponse {
    query: String,
    status: String,
    country: Option<String>,
    #[serde(rename = "regionName")]
    region_name: Option<String>,
    city: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
    isp: Option<String>,
    org: Option<String>,
}

#[async_trait]
impl GeoLocator for IpApiGeoLocator {
    async fn locate_hops(&self, traces: &[PathTrace]) -> Result<HashMap<String, Location>, String> {
        let ips = traces
            .iter()
            .flat_map(|trace| trace.hops.iter())
            .filter_map(|hop| hop.ip.as_deref())
            .filter(|ip| !is_private_or_loopback(ip))
            .take(100)
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();

        if ips.is_empty() {
            return Ok(HashMap::new());
        }

        let client = reqwest::Client::new();
        let response = client
            .post("http://ip-api.com/batch?fields=status,query,country,regionName,city,lat,lon,isp,org")
            .json(&ips)
            .timeout(Duration::from_secs(3))
            .send()
            .await
            .map_err(|error| format!("geo lookup failed: {error}"))?;

        let locations = response
            .json::<Vec<IpApiResponse>>()
            .await
            .map_err(|error| format!("geo response parse failed: {error}"))?;

        Ok(locations
            .into_iter()
            .filter(|location| location.status == "success")
            .map(|location| {
                (
                    location.query,
                    Location {
                        country: location.country,
                        region_name: location.region_name,
                        city: location.city,
                        lat: location.lat,
                        lon: location.lon,
                        isp: location.isp,
                        org: location.org,
                    },
                )
            })
            .collect())
    }
}

fn is_private_or_loopback(ip: &str) -> bool {
    ip.parse::<IpAddr>()
        .map(|ip| match ip {
            IpAddr::V4(ip) => ip.is_private() || ip.is_loopback() || ip.is_link_local(),
            IpAddr::V6(ip) => {
                ip.is_loopback() || ip.is_unique_local() || ip.is_unicast_link_local()
            }
        })
        .unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::network::{Hop, PathTrace};

    #[test]
    fn private_loopback_link_local_and_invalid_ips_are_filtered() {
        for ip in ["10.0.0.1", "172.16.0.1", "192.168.1.1", "127.0.0.1", "169.254.1.1", "::1", "fc00::1", "fe80::1", "not-an-ip"] {
            assert!(is_private_or_loopback(ip), "{ip} should be filtered");
        }
    }

    #[test]
    fn public_ips_are_not_filtered() {
        for ip in ["1.1.1.1", "8.8.8.8", "2606:4700:4700::1111"] {
            assert!(!is_private_or_loopback(ip), "{ip} should be public");
        }
    }

    #[tokio::test]
    async fn locate_hops_returns_empty_without_public_ips() {
        let locator = IpApiGeoLocator;
        let traces = vec![PathTrace {
            target: "router".to_string(),
            hops: vec![Hop {
                ttl: 1,
                ip: Some("192.168.1.1".to_string()),
                rtt_ms: Some(1.0),
                location: None,
            }],
            error: None,
        }];

        let locations = locator.locate_hops(&traces).await.unwrap();

        assert!(locations.is_empty());
    }
}
