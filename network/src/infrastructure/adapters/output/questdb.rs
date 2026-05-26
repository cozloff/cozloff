use crate::{application::ports::output::NetworkHopRepository, domain::network::PathTrace};
use async_trait::async_trait;
use sqlx::{PgPool, postgres::PgPoolOptions};

#[derive(Clone)]
pub struct QuestDbNetworkHopRepository {
    pool: PgPool,
}

impl QuestDbNetworkHopRepository {
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        ensure_schema(&pool).await?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl NetworkHopRepository for QuestDbNetworkHopRepository {
    async fn save_path_traces(&self, traces: &[PathTrace]) -> Result<(), String> {
        for trace in traces {
            if trace.hops.is_empty() {
                insert_hop(&self.pool, trace, None).await?;
                continue;
            }

            for hop in &trace.hops {
                insert_hop(&self.pool, trace, Some(hop)).await?;
            }
        }

        Ok(())
    }
}

async fn insert_hop(
    pool: &PgPool,
    trace: &PathTrace,
    hop: Option<&crate::domain::network::Hop>,
) -> Result<(), String> {
    let ttl = hop.map(|hop| hop.ttl as i32);
    let ip = hop.and_then(|hop| hop.ip.as_deref());
    let rtt_ms = hop.and_then(|hop| hop.rtt_ms).map(f64::from);
    let location = hop.and_then(|hop| hop.location.as_ref());

    sqlx::query(
        r#"
        INSERT INTO network_hops (
            ts,
            target,
            ttl,
            ip,
            rtt_ms,
            country,
            region_name,
            city,
            lat,
            lon,
            isp,
            org,
            error
        ) VALUES (
            now(),
            $1,
            $2,
            $3,
            $4,
            $5,
            $6,
            $7,
            $8,
            $9,
            $10,
            $11,
            $12
        )
        "#,
    )
    .bind(&trace.target)
    .bind(ttl)
    .bind(ip)
    .bind(rtt_ms)
    .bind(location.and_then(|location| location.country.as_deref()))
    .bind(location.and_then(|location| location.region_name.as_deref()))
    .bind(location.and_then(|location| location.city.as_deref()))
    .bind(location.and_then(|location| location.lat))
    .bind(location.and_then(|location| location.lon))
    .bind(location.and_then(|location| location.isp.as_deref()))
    .bind(location.and_then(|location| location.org.as_deref()))
    .bind(trace.error.as_deref())
    .execute(pool)
    .await
    .map_err(|error| format!("failed to insert network hop: {error}"))?;

    Ok(())
}

async fn ensure_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS network_hops (
            ts TIMESTAMP,
            target SYMBOL,
            ttl INT,
            ip STRING,
            rtt_ms DOUBLE,
            country SYMBOL,
            region_name SYMBOL,
            city SYMBOL,
            lat DOUBLE,
            lon DOUBLE,
            isp STRING,
            org STRING,
            error STRING
        ) TIMESTAMP(ts) PARTITION BY DAY WAL
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
