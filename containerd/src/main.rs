use containerd_client::{connect, services::v1::version_client::VersionClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    query_version().await?;
    Ok(())
}

async fn query_version() -> anyhow::Result<()> {
    let channel = connect("/run/containerd/containerd.sock").await?;

    let mut client = VersionClient::new(channel);
    let resp = client.version(()).await?;

    println!("Response: {:?}", resp.get_ref());

    Ok(())
}