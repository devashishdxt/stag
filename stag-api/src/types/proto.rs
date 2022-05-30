#![allow(missing_docs)]

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
