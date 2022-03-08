use anyhow::{ensure, Context, Error};

use cosmos_sdk_proto::ibc::core::client::v1::Height;

pub trait IHeight: Sized {
    fn new(revision_number: u64, revision_height: u64) -> Self;

    fn zero() -> Self {
        Self::new(0, 0)
    }

    fn checked_add(self, rhs: u64) -> Option<Self>;

    fn to_string(&self) -> String;

    fn from_str(height: &str) -> Result<Self, Error>;
}

impl IHeight for Height {
    fn new(revision_number: u64, revision_height: u64) -> Self {
        Self {
            revision_number,
            revision_height,
        }
    }

    fn checked_add(self, rhs: u64) -> Option<Self> {
        let revision_number = self.revision_number;
        let revision_height = self.revision_height.checked_add(rhs)?;

        Some(Self {
            revision_number,
            revision_height,
        })
    }

    fn to_string(&self) -> String {
        format!("{}-{}", self.revision_number, self.revision_height)
    }

    fn from_str(height: &str) -> Result<Self, Error> {
        let split: Vec<&str> = height.split('-').collect();

        ensure!(
            split.len() == 2,
            "height should be of format {{revision_number}}-{{revision_height}}"
        );

        Ok(Height {
            revision_number: split[0].parse().context("invalid revision number")?,
            revision_height: split[1].parse().context("invalid revision height")?,
        })
    }
}
