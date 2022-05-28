// Some part of this file is taken from the original project. https://github.com/informalsystems/tendermint-rs
// Copyright © 2020 Informal Systems

use std::{sync::Arc, time::Duration};

use anyhow::{anyhow, bail, Context, Result};
use tendermint::{block::Height, trust_threshold::TrustThresholdFraction, Hash, Time};
use tendermint_light_client::{
    components::{
        scheduler::basic_bisecting_schedule,
        verifier::{ProdVerifier, Verdict, Verifier},
    },
    light_client::Options,
    state::State,
    store::memory::MemoryStore,
    types::{LightBlock, Status},
};
use tokio::sync::Mutex;
use url::Url;

use crate::{time_util::now_utc, types::chain_state::ChainState};

use super::tendermint_client::TendermintClient;

/// Tendermint light client
pub struct LightClient<T>
where
    T: TendermintClient,
{
    url: Url,
    state: Arc<Mutex<State>>,
    tendermint_client: T,
    verifier: ProdVerifier,
    options: Options,
}

impl<T> LightClient<T>
where
    T: TendermintClient,
{
    /// Creates a new tendermint light client
    pub async fn new(url: Url, tendermint_client: T, chain_state: &ChainState) -> Result<Self> {
        let options = Options {
            trust_threshold: TrustThresholdFraction::new(
                *chain_state.config.trust_level.numer(),
                *chain_state.config.trust_level.denom(),
            )
            .unwrap(),
            trusting_period: chain_state.config.trusting_period,
            clock_drift: chain_state.config.max_clock_drift,
        };

        let this = LightClient {
            url,
            state: Arc::new(Mutex::new(State::new(MemoryStore::new()))),
            tendermint_client,
            verifier: Default::default(),
            options,
        };

        this.trust_block(
            chain_state.config.trusted_height,
            chain_state.config.trusted_hash,
        )
        .await?;

        Ok(this)
    }

    /// Runs the light client verification process until the latest block
    pub async fn verify_to_highest(&self) -> Result<LightBlock> {
        let target_block = self.tendermint_client.light_block(&self.url, None).await?;
        let target_height = target_block.height();

        let mut state = self.state.lock().await;

        // Let's first look in the store to see whether
        // we have already successfully verified this block.
        if let Some(light_block) = state.light_store.get_trusted_or_verified(target_height) {
            return Ok(light_block);
        }

        // Get the highest trusted state
        let highest = state
            .light_store
            .highest_trusted_or_verified()
            .context("no initial trusted state in light client")?;

        if target_height >= highest.height() {
            // Perform forward verification with bisection
            self.verify_forward(target_height, &mut state).await
        } else {
            // Perform sequential backward verification
            self.verify_backward(target_height, &mut state).await
        }
    }

    /// Adds a block to trusted state
    async fn trust_block(&self, trusted_height: u32, trusted_hash: [u8; 32]) -> Result<()> {
        let trusted_block = self
            .tendermint_client
            .light_block(&self.url, Some(trusted_height))
            .await?;

        if trusted_block.height() != trusted_height.into() {
            bail!(
                "Trusted block height [{}] does not match trusted height [{}]",
                trusted_block.height(),
                trusted_height
            );
        }

        let header_hash = trusted_block.signed_header.header.hash();
        let trusted_hash = Hash::Sha256(trusted_hash);

        if header_hash != trusted_hash {
            bail!(
                "Trusted block hash [{}] does not match trusted hash [{}]",
                header_hash,
                trusted_hash
            );
        }

        self.trust_light_block(trusted_block).await
    }

    /// Set the given light block as the initial trusted state.
    async fn trust_light_block(&self, trusted_block: LightBlock) -> Result<()> {
        self.validate(&trusted_block)?;

        let mut state = self.state.lock().await;

        // TODO(liamsi, romac): it is unclear if this should be Trusted or only Verified
        state.light_store.insert(trusted_block, Status::Trusted);

        Ok(())
    }

    /// Validates a light block
    fn validate(&self, light_block: &LightBlock) -> Result<()> {
        let now = now()?;

        if !is_within_trust_period(light_block, self.options.trusting_period, now) {
            bail!(
                "trusted state [{}] is outside trusting period. Options: {}",
                light_block.height(),
                self.options
            );
        }

        is_from_past(light_block, self.options.clock_drift, now)?;
        validator_sets_match(light_block)?;
        next_validators_match(light_block)?;

        Ok(())
    }

    async fn verify_forward(&self, target_height: Height, state: &mut State) -> Result<LightBlock> {
        let mut current_height = target_height;

        loop {
            let now = now()?;

            // Get the latest trusted state
            let trusted_block = state
                .light_store
                .highest_trusted_or_verified()
                .ok_or_else(|| anyhow!("no initial trusted state"))?;

            if target_height < trusted_block.height() {
                bail!(
                    "target height [{}] is lower than trusted state [{}]",
                    target_height,
                    trusted_block.height()
                );
            }

            // Check invariant [LCV-INV-TP.1]
            if !is_within_trust_period(&trusted_block, self.options.trusting_period, now) {
                bail!(
                    "trusted state [{}] is outside trusting period. Options: {}",
                    trusted_block.height(),
                    self.options
                );
            }

            // Log the current height as a dependency of the block at the target height
            state.trace_block(target_height, current_height);

            // If the trusted state is now at a height equal to the target height, we are done.
            // [LCV-DIST-LIFE.1]
            if target_height == trusted_block.height() {
                return Ok(trusted_block);
            }

            // Fetch the block at the current height from the light store if already present,
            // or from the primary peer otherwise.
            let (current_block, status) = self.get_or_fetch_block(current_height, state).await?;

            // Validate and verify the current block
            let verdict = self.verifier.verify(
                current_block.as_untrusted_state(),
                trusted_block.as_trusted_state(),
                &self.options,
                now,
            );

            match verdict {
                Verdict::Success => {
                    // Verification succeeded, add the block to the light store with
                    // the `Verified` status or higher if already trusted.
                    let new_status = Status::most_trusted(Status::Verified, status);
                    state.light_store.update(&current_block, new_status);
                }
                Verdict::Invalid(e) => {
                    // Verification failed, add the block to the light store with `Failed` status,
                    // and abort.
                    state.light_store.update(&current_block, Status::Failed);

                    bail!("invalid light block: {}", e);
                }
                Verdict::NotEnoughTrust(_) => {
                    // The current block cannot be trusted because of a missing overlap in the
                    // validator sets. Add the block to the light store with
                    // the `Unverified` status. This will engage bisection in an
                    // attempt to raise the height of the highest trusted state
                    // until there is enough overlap.
                    state.light_store.update(&current_block, Status::Unverified);
                }
            }

            // Compute the next height to fetch and verify
            current_height =
                basic_bisecting_schedule(state.light_store.as_ref(), current_height, target_height);
        }
    }

    async fn verify_backward(
        &self,
        target_height: Height,
        state: &mut State,
    ) -> Result<LightBlock> {
        let root = state
            .light_store
            .highest_trusted_or_verified()
            .context("no initial trusted state")?;

        assert!(root.height() >= target_height);

        // Check invariant [LCV-INV-TP.1]
        if !is_within_trust_period(&root, self.options.trusting_period, now()?) {
            bail!(
                "trusted state [{}] is outside trusting period. Options: {}",
                root.height(),
                self.options
            );
        }

        // Compute a range of `Height`s from `trusted_height - 1` to `target_height`, inclusive.
        let range = (target_height.value()..root.height().value()).rev();
        let heights = range.map(|h| Height::try_from(h).unwrap());

        let mut latest = root;

        for height in heights {
            let (current, _status) = self.get_or_fetch_block(height, state).await?;

            let latest_last_block_id = latest
                .signed_header
                .header
                .last_block_id
                .ok_or_else(|| anyhow!("missing last block id: {}", latest.height()))?;

            let current_hash = current.signed_header.header.hash();

            if current_hash != latest_last_block_id.hash {
                bail!(
                    "invalid adjacent headers: {} and {}",
                    current_hash,
                    latest_last_block_id.hash
                );
            }

            // `latest` and `current` are linked together by `last_block_id`,
            // therefore it is not relevant which we verified first.
            // For consistency, we say that `latest` was verifed using
            // `current` so that the trace is always pointing down the chain.
            state.light_store.insert(current.clone(), Status::Trusted);
            state.light_store.insert(latest.clone(), Status::Trusted);
            state.trace_block(latest.height(), current.height());

            latest = current;
        }

        // We reached the target height.
        assert_eq!(latest.height(), target_height);

        Ok(latest)
    }

    /// Look in the light store for a block from the given peer at the given height,
    /// which has not previously failed verification (ie. its status is not `Failed`).
    ///
    /// If one cannot be found, fetch the block from the given peer and store
    /// it in the light store with `Unverified` status.
    ///
    /// ## Postcondition
    /// - The provider of block that is returned matches the given peer.
    async fn get_or_fetch_block(
        &self,
        height: Height,
        state: &mut State,
    ) -> Result<(LightBlock, Status)> {
        let block = state.light_store.get_non_failed(height);

        if let Some(block) = block {
            return Ok(block);
        }

        let block = self
            .tendermint_client
            .light_block(&self.url, Some(height.value().try_into()?))
            .await?;

        state.light_store.insert(block.clone(), Status::Unverified);

        Ok((block, Status::Unverified))
    }
}

/// Whether or not the given block is within the given trusting period,
/// relative to the given time.
fn is_within_trust_period(light_block: &LightBlock, trusting_period: Duration, now: Time) -> bool {
    let header_time = light_block.signed_header.header.time;
    match now - trusting_period {
        Ok(start) => header_time > start,
        Err(_) => false,
    }
}

fn is_from_past(light_block: &LightBlock, clock_drift: Duration, now: Time) -> Result<()> {
    let untrusted_header_time = light_block.signed_header.header.time;
    let drifted = (now + clock_drift).context("time overflow")?;

    if untrusted_header_time < drifted {
        Ok(())
    } else {
        Err(anyhow!(
            "untrusted header time is in the future: {} >= {}",
            untrusted_header_time,
            now
        ))
    }
}

/// Compare the provided validator_set_hash against the hash produced from hashing the validator
/// set.
fn validator_sets_match(light_block: &LightBlock) -> Result<()> {
    let validators_hash = light_block.validators.hash();
    let header_validators_hash = light_block.signed_header.header.validators_hash;

    if header_validators_hash == validators_hash {
        Ok(())
    } else {
        Err(anyhow!(
            "validator set hash mismatch: {} != {}",
            header_validators_hash,
            validators_hash
        ))
    }
}

/// Check that the hash of the next validator set in the header match the actual one.
fn next_validators_match(light_block: &LightBlock) -> Result<()> {
    let next_validators_hash = light_block.next_validators.hash();
    let header_validators_hash = light_block.signed_header.header.validators_hash;

    if header_validators_hash == next_validators_hash {
        Ok(())
    } else {
        Err(anyhow!(
            "next validator set hash mismatch: {} != {}",
            header_validators_hash,
            next_validators_hash
        ))
    }
}

/// Returns current time
fn now() -> Result<Time> {
    let offset_date_time = now_utc();

    Time::from_unix_timestamp(
        offset_date_time.timestamp(),
        offset_date_time.timestamp_subsec_nanos(),
    )
    .map_err(|_| anyhow!("time overflow"))
}
