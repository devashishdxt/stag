use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "mnemonic-signer")] {
            tonic_build::configure().compile(
                &[
                    "./proto/core.proto",
                    "./proto/transfer.proto",
                    "./proto/ica/bank.proto",
                    "./proto/ica/staking.proto",
                    "./proto/query.proto",
                    "./proto/mnemonic_signer.proto"
                ],
                &["proto"],
            )?;
        } else {
            tonic_build::configure().compile(
                &[
                    "./proto/core.proto",
                    "./proto/transfer.proto",
                    "./proto/ica/bank.proto",
                    "./proto/ica/staking.proto",
                    "./proto/query.proto",
                ],
                &["proto"],
            )?;
        }
    }

    Ok(())
}
