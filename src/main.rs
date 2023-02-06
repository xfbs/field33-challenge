use anyhow::Result;
use clap::Parser;
use field33_challenge::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let options = Options::parse();
    options.run().await?;
    Ok(())
}
