use crate::models::{Location, PathTrace};
use serde::Deserialize;
use std::{collections::HashMap, net::IpAddr, time::Duration};

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

pub async fn locate_hops(traces: &[PathTrace]) -> HashMap<String, Location> {
    let ips = traces
        .iter()
        .flat_map(|trace| trace.hops.iter())
        .filter_map(|hop| hop.ip.as_deref())
        .filter(|ip| !is_private_or_loopback(ip))
        .take(100)
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    let client = reqwest::Client::new();
    let Ok(response) = client
        .post("http://ip-api.com/batch?fields=status,query,country,regionName,city,lat,lon,isp,org")
        .json(&ips)
        .timeout(Duration::from_secs(3))
        .send()
        .await
    else {
        return HashMap::new();
    };

    let Ok(locations) = response.json::<Vec<IpApiResponse>>().await else {
        return HashMap::new();
    };

    locations
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
        .collect()
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
