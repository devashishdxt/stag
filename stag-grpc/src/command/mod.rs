mod subcommand;

use anyhow::Result;
use clap::Parser;

use self::subcommand::SubCommand;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Command {
    #[clap(subcommand)]
    subcommand: SubCommand,
}

impl Command {
    pub async fn run(self) -> Result<()> {
        self.subcommand.run().await
    }
}
