//! Traits for performing different operations related to solo machine.
mod chain_service;
mod ibc_service;

pub use self::{chain_service::*, ibc_service::*};
