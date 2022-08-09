use anyhow::Result;
use clap::Parser;
use stag_grpc::Server;
use tracing::info;

#[derive(Parser, Debug)]
pub enum Command {
    /// Start Stag gRPC server
    Start {
        /// gRPC server port
        #[clap(short, long, default_value = "8000")]
        port: u16,
        /// gRPC server database uri
        #[clap(short, long)]
        #[cfg_attr(feature = "sqlite-storage", clap(default_value = "sqlite::memory:"))]
        db_uri: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let command = Command::parse();

    tracing_subscriber::fmt().init();

    match command {
        Command::Start { port, db_uri } => {
            info!("starting stag grpc server on port {}", port);
            let server = Server::new(format!("[::1]:{port}").parse().unwrap(), db_uri);
            server.run().await
        }
    }
}
