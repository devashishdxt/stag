use anyhow::Result;
use primitive_types::U256;

use crate::{ChainConfig, ChainId, Identifier, Signer};

#[derive(Debug)]
pub struct StagConfig {}

#[derive(Debug)]
pub struct Stag<S>
where
    S: Signer,
{
    signer: S,
    config: StagConfig,
}

/// IBC Solo Machine
impl<S> Stag<S>
where
    S: Signer,
{
    /// Creates a new instance of IBC Solo Machine
    pub fn new(signer: S, config: StagConfig) -> Self {
        Self { signer, config }
    }

    /// Adds metadata of a given chain
    pub async fn add_chain(&self, chain_id: ChainId, chain_config: ChainConfig) -> Result<()> {
        todo!()
    }

    /// Establishes connection with given chain
    pub async fn connect(
        &self,
        chain_id: ChainId,
        request_id: Option<String>,
        memo: String,
        force: bool,
    ) -> Result<()> {
        todo!()
    }

    /// Mints tokens on given chain
    pub async fn mint(
        &self,
        chain_id: ChainId,
        request_id: Option<String>,
        amount: U256,
        denom: Identifier,
        receiver: Option<String>,
        memo: String,
    ) -> Result<String> {
        todo!()
    }

    /// Burns tokens on given chain
    pub async fn burn(
        &self,
        chain_id: ChainId,
        request_id: Option<String>,
        amount: U256,
        denom: Identifier,
        memo: String,
    ) -> Result<String> {
        todo!()
    }
}
