use field33_challenge::*;
use clap::Parser;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let options = Options::parse();
}
