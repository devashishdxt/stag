use std::path::Path;

use anyhow::Result;
use clap::Subcommand;
use primitive_types::U256;
use stag_api::types::ics::core::ics24_host::identifier::{ChainId, Identifier};

use crate::{u256::U256Parser, util::stag};

#[derive(Debug, Subcommand)]
pub enum StakingCommand {
    /// Delegate tokens from ICA (Interchain Account) on host chain to given validator address
    Delegate {
        /// Chain ID
        chain_id: ChainId,
        /// An optional request ID for tracking purposes
        #[clap(long)]
        request_id: Option<String>,
        /// Validator address on IBC enabled chain
        validator_address: String,
        /// Amount of tokens to delegate
        #[clap(value_parser = U256Parser)]
        amount: U256,
        /// Denom of tokens to delegate
        denom: Identifier,
        /// Memo value to be used in cosmos sdk transaction
        #[clap(long)]
        memo: Option<String>,
    },
    /// Un-delegate tokens to ICA (Interchain Account) on host chain from given validator address
    Undelegate {
        /// Chain ID
        chain_id: ChainId,
        /// An optional request ID for tracking purposes
        #[clap(long)]
        request_id: Option<String>,
        /// Validator address on IBC enabled chain
        validator_address: String,
        /// Amount of tokens to un-delegate
        #[clap(value_parser = U256Parser)]
        amount: U256,
        /// Denom of tokens to un-delegate
        denom: Identifier,
        /// Memo value to be used in cosmos sdk transaction
        #[clap(long)]
        memo: Option<String>,
    },
}

impl StakingCommand {
    pub async fn run(self, signer: impl AsRef<Path>, db_uri: &str) -> Result<()> {
        match self {
            Self::Delegate {
                chain_id,
                request_id,
                validator_address,
                amount,
                denom,
                memo,
            } => {
                stag(signer, db_uri)
                    .await?
                    .ica_delegate(
                        chain_id,
                        request_id,
                        validator_address.clone(),
                        amount,
                        denom.clone(),
                        memo.unwrap_or_default(),
                    )
                    .await?;

                println!(
                    "successfully delegated {} {} to {}",
                    amount, denom, validator_address
                );

                Ok(())
            }
            Self::Undelegate {
                chain_id,
                request_id,
                validator_address,
                amount,
                denom,
                memo,
            } => {
                stag(signer, db_uri)
                    .await?
                    .ica_undelegate(
                        chain_id,
                        request_id,
                        validator_address.clone(),
                        amount,
                        denom.clone(),
                        memo.unwrap_or_default(),
                    )
                    .await?;

                println!(
                    "successfully un-delegated {} {} from {}",
                    amount, denom, validator_address
                );

                Ok(())
            }
        }
    }
}
