use std::path::Path;

use anyhow::Result;
use clap::Subcommand;
use primitive_types::U256;
use stag_api::types::ics::core::ics24_host::identifier::{ChainId, Identifier};

use crate::{u256::U256Parser, util::stag};

#[derive(Debug, Subcommand)]
pub enum TransferCommand {
    /// Mint tokens on IBC enabled chain
    Mint {
        /// Chain ID
        chain_id: ChainId,
        /// An optional request ID for tracking purposes
        #[clap(long)]
        request_id: Option<String>,
        /// Amount of tokens to be minted
        #[clap(value_parser = U256Parser)]
        amount: U256,
        /// Denom of tokens to be minted
        denom: Identifier,
        /// Receiver address on IBC enabled chain (if this is not provided, tokens will be sent to signer's address)
        #[clap(short, long)]
        receiver: Option<String>,
        /// Memo value to be used in cosmos sdk transaction
        #[clap(long)]
        memo: Option<String>,
    },
    /// Burn tokens from IBC enabled chain
    Burn {
        /// Chain ID
        chain_id: ChainId,
        /// An optional request ID for tracking purposes
        #[clap(long)]
        request_id: Option<String>,
        /// Amount of tokens to be burnt
        #[clap(value_parser = U256Parser)]
        amount: U256,
        /// Denom of tokens to be burnt
        denom: Identifier,
        /// Memo value to be used in cosmos sdk transaction
        #[clap(long)]
        memo: Option<String>,
    },
}

impl TransferCommand {
    pub async fn run(self, signer: impl AsRef<Path>, db_uri: &str) -> Result<()> {
        match self {
            Self::Mint {
                chain_id,
                request_id,
                amount,
                denom,
                receiver,
                memo,
            } => {
                stag(signer, db_uri)
                    .await?
                    .mint(
                        chain_id.clone(),
                        request_id,
                        amount,
                        denom.clone(),
                        receiver,
                        memo.unwrap_or_default(),
                    )
                    .await?;

                println!("successfully minted {} {} on {}", amount, denom, chain_id);

                Ok(())
            }
            Self::Burn {
                chain_id,
                request_id,
                amount,
                denom,
                memo,
            } => {
                stag(signer, db_uri)
                    .await?
                    .burn(
                        chain_id.clone(),
                        request_id,
                        amount,
                        denom.clone(),
                        memo.unwrap_or_default(),
                    )
                    .await?;

                println!("successfully burnt {} {} from {}", amount, denom, chain_id);

                Ok(())
            }
        }
    }
}
