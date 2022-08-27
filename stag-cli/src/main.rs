pub mod command;
pub mod signer_config;
pub mod u256;
pub mod util;

use clap::Parser;

use self::command::Command;

#[tokio::main]
async fn main() {
    if let Err(err) = Command::parse().run().await {
        eprintln!("{:?}", err);
    }
}
