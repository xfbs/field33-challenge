use clap::Parser;
use std::net::SocketAddr;
use url::Url;

#[derive(Parser, Clone, Debug)]
pub struct Options {
    #[clap(long, short, default_value = "bolt://localhost:7687")]
    pub database: Url,
    #[clap(long, short, default_value = "neo4j")]
    pub username: String,
    #[clap(long, short, default_value = "localpw")]
    pub password: String,
    #[clap(long, short, default_value = "0.0.0.0:8000")]
    pub listen: SocketAddr,
}
