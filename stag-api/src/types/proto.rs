pub mod cosmos {
    pub mod auth {
        pub mod v1beta1 {
            tonic::include_proto!("cosmos.auth.v1beta1");
        }
    }

    pub mod bank {
        pub mod v1beta1 {
            tonic::include_proto!("cosmos.bank.v1beta1");
        }
    }

    pub mod staking {
        pub mod v1beta1 {
            tonic::include_proto!("cosmos.staking.v1beta1");
        }
    }
}

#[cfg(feature = "ethermint")]
pub mod ethermint {
    pub mod crypto {
        pub mod v1 {
            pub mod ethsecp256k1 {
                tonic::include_proto!("ethermint.crypto.v1.ethsecp256k1");
            }
        }
    }

    pub mod types {
        pub mod v1 {
            tonic::include_proto!("ethermint.types.v1");
        }
    }
}

pub mod ibc {
    pub mod lightclients {
        pub mod solomachine {
            pub mod v2 {
                tonic::include_proto!("ibc.lightclients.solomachine.v2");
            }
        }
    }
}
