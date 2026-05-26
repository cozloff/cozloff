use crate::{
    application::ports::output::NetworkProbe,
    domain::network::{Hop, PathTrace},
};
use async_trait::async_trait;
use std::net::IpAddr;
use tokio::process::Command;

#[derive(Clone, Default)]
pub struct CommandNetworkProbe;

#[async_trait]
impl NetworkProbe for CommandNetworkProbe {
    async fn trace_target(&self, target: String, max_hops: u8) -> Result<PathTrace, String> {
        Ok(match run_traceroute(&target, max_hops).await {
            Some(trace) => trace,
            None => run_ping(&target).await,
        })
    }
}

async fn run_traceroute(target: &str, max_hops: u8) -> Option<PathTrace> {
    let traceroute = Command::new("traceroute")
        .args([
            "-n",
            "-q",
            "1",
            "-w",
            "1",
            "-m",
            &max_hops.to_string(),
            target,
        ])
        .output()
        .await;

    match traceroute {
        Ok(output) if output.status.success() || !output.stdout.is_empty() => Some(PathTrace {
            target: target.to_string(),
            hops: parse_traceroute(&String::from_utf8_lossy(&output.stdout)),
            error: (!output.status.success())
                .then(|| String::from_utf8_lossy(&output.stderr).trim().to_string())
                .filter(|error| !error.is_empty()),
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            run_tracepath(target, max_hops).await
        }
        Ok(output) => Some(PathTrace {
            target: target.to_string(),
            hops: Vec::new(),
            error: Some(String::from_utf8_lossy(&output.stderr).trim().to_string()),
        }),
        Err(error) => Some(PathTrace {
            target: target.to_string(),
            hops: Vec::new(),
            error: Some(format!("failed to run traceroute: {error}")),
        }),
    }
}

async fn run_tracepath(target: &str, max_hops: u8) -> Option<PathTrace> {
    let tracepath = Command::new("tracepath")
        .args(["-n", "-m", &max_hops.to_string(), target])
        .output()
        .await;

    match tracepath {
        Ok(output) if output.status.success() || !output.stdout.is_empty() => Some(PathTrace {
            target: target.to_string(),
            hops: parse_tracepath(&String::from_utf8_lossy(&output.stdout)),
            error: (!output.status.success())
                .then(|| String::from_utf8_lossy(&output.stderr).trim().to_string())
                .filter(|error| !error.is_empty()),
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Ok(output) => Some(PathTrace {
            target: target.to_string(),
            hops: Vec::new(),
            error: Some(String::from_utf8_lossy(&output.stderr).trim().to_string()),
        }),
        Err(error) => Some(PathTrace {
            target: target.to_string(),
            hops: Vec::new(),
            error: Some(format!("failed to run tracepath: {error}")),
        }),
    }
}

fn parse_traceroute(output: &str) -> Vec<Hop> {
    output
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let ttl = parts.next()?.parse().ok()?;
            let ip = parts
                .find(|part| part.parse::<IpAddr>().is_ok())
                .map(str::to_string);
            let rtt_ms = parts.find_map(|part| part.parse::<f32>().ok());

            Some(Hop {
                ttl,
                ip,
                rtt_ms,
                location: None,
            })
        })
        .collect()
}

fn parse_tracepath(output: &str) -> Vec<Hop> {
    output
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let ttl = parts.next()?.trim_end_matches(':').parse().ok()?;
            let ip = parts
                .find(|part| part.parse::<IpAddr>().is_ok())
                .map(str::to_string);
            let rtt_ms = parts.find_map(|part| part.trim_end_matches("ms").parse::<f32>().ok());

            Some(Hop {
                ttl,
                ip,
                rtt_ms,
                location: None,
            })
        })
        .collect()
}

async fn run_ping(target: &str) -> PathTrace {
    let ping = Command::new("ping")
        .args(["-c", "1", "-W", "1", target])
        .output()
        .await;

    match ping {
        Ok(output) if output.status.success() || !output.stdout.is_empty() => PathTrace {
            target: target.to_string(),
            hops: parse_ping(&String::from_utf8_lossy(&output.stdout)),
            error: (!output.status.success())
                .then(|| String::from_utf8_lossy(&output.stderr).trim().to_string())
                .filter(|error| !error.is_empty()),
        },
        Ok(output) => PathTrace {
            target: target.to_string(),
            hops: Vec::new(),
            error: Some(String::from_utf8_lossy(&output.stderr).trim().to_string()),
        },
        Err(error) => PathTrace {
            target: target.to_string(),
            hops: Vec::new(),
            error: Some(format!("failed to run ping: {error}")),
        },
    }
}

fn parse_ping(output: &str) -> Vec<Hop> {
    let Some(line) = output.lines().find(|line| line.contains(" from ")) else {
        return Vec::new();
    };

    let ip = line.split_whitespace().find_map(|part| {
        part.trim_end_matches(':')
            .trim_start_matches('(')
            .trim_end_matches(')')
            .parse::<IpAddr>()
            .ok()
            .map(|ip| ip.to_string())
    });
    let rtt_ms = line
        .split_whitespace()
        .find_map(|part| part.strip_prefix("time=")?.parse::<f32>().ok());

    vec![Hop {
        ttl: 0,
        ip,
        rtt_ms,
        location: None,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_traceroute_extracts_ttl_ip_and_rtt() {
        let output = "\
traceroute to 8.8.8.8 (8.8.8.8), 30 hops max
 1  192.168.1.1  1.234 ms
 2  *
 3  8.8.8.8  12.345 ms
";

        let hops = parse_traceroute(output);

        assert_eq!(hops.len(), 3);
        assert_eq!(hops[0].ttl, 1);
        assert_eq!(hops[0].ip, Some("192.168.1.1".to_string()));
        assert_eq!(hops[0].rtt_ms, Some(1.234));
        assert_eq!(hops[1].ttl, 2);
        assert_eq!(hops[1].ip, None);
        assert_eq!(hops[2].ip, Some("8.8.8.8".to_string()));
    }

    #[test]
    fn parse_tracepath_extracts_colon_ttl_ip_and_rtt() {
        let output = "\
 1?: [LOCALHOST]                      pmtu 1500
 1:  192.168.1.1                                           1.100ms
 2:  8.8.8.8                                               8.250ms
";

        let hops = parse_tracepath(output);

        assert_eq!(hops.len(), 2);
        assert_eq!(hops[0].ttl, 1);
        assert_eq!(hops[0].ip, Some("192.168.1.1".to_string()));
        assert_eq!(hops[0].rtt_ms, Some(1.1));
        assert_eq!(hops[1].ttl, 2);
        assert_eq!(hops[1].ip, Some("8.8.8.8".to_string()));
        assert_eq!(hops[1].rtt_ms, Some(8.25));
    }

    #[test]
    fn parse_ping_extracts_reply_ip_and_time() {
        let output = "\
PING dns.google (8.8.8.8) 56(84) bytes of data.
64 bytes from dns.google (8.8.8.8): icmp_seq=1 ttl=117 time=13.7 ms
";

        let hops = parse_ping(output);

        assert_eq!(hops.len(), 1);
        assert_eq!(hops[0].ttl, 0);
        assert_eq!(hops[0].ip, Some("8.8.8.8".to_string()));
        assert_eq!(hops[0].rtt_ms, Some(13.7));
    }

    #[test]
    fn parse_ping_returns_no_hops_without_reply_line() {
        assert!(parse_ping("PING example.com\n").is_empty());
    }
}
