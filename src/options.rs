use clap::Parser;
use std::net::SocketAddr;
use url::Url;

/// Backend for abstract graph editor
#[derive(Parser, Clone, Debug)]
#[clap(version)]
pub struct Options {
    /// URL of database
    #[clap(long, short, default_value = "bolt://127.0.0.1:7687")]
    pub database: Url,
    /// Username of database
    #[clap(long, short, default_value = "neo4j")]
    pub username: String,
    /// Password of database
    #[clap(long, short, default_value = "localpw")]
    pub password: String,
    /// Address to listen on
    #[clap(long, short, default_value = "0.0.0.0:8000")]
    pub listen: SocketAddr,
}
