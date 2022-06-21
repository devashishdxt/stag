mod command;
mod service;

use anyhow::Result;
use clap::Parser;

use self::command::Command;

#[cfg(not(feature = "sqlite-storage"))]
compile_error!("`sqlite-storage` feature is not enabled");

#[tokio::main]
async fn main() -> Result<()> {
    Command::parse().run().await
}
