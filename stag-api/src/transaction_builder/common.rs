use anyhow::{Context, Result};
use time::OffsetDateTime;

pub fn to_u64_timestamp(timestamp: OffsetDateTime) -> Result<u64> {
    timestamp
        .unix_timestamp()
        .try_into()
        .context("unable to convert unix timestamp to u64")
}
