pub mod ica {
    pub mod bank {
        tonic::include_proto!("ica.bank");
    }

    pub mod staking {
        tonic::include_proto!("ica.staking");
    }
}

pub mod core {
    tonic::include_proto!("core");
}

#[cfg(feature = "mnemonic-signer")]
pub mod mnemonic_signer {
    tonic::include_proto!("mnemonic_signer");
}

pub mod query {
    tonic::include_proto!("query");
}

pub mod transfer {
    tonic::include_proto!("transfer");
}
