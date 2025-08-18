use server::SmmServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = SmmServer::new();
    server.run().await?;
    Ok(())
}
