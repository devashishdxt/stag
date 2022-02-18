use anyhow::{Context, Result};
use chrono::{DateTime, Utc};

pub fn to_u64_timestamp(timestamp: DateTime<Utc>) -> Result<u64> {
    timestamp
        .timestamp()
        .try_into()
        .context("unable to convert unix timestamp to u64")
}
