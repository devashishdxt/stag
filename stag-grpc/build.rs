use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    tonic_build::configure().build_client(false).compile(
        &[
            "./proto/core.proto",
            "./proto/transfer.proto",
            "./proto/ica/bank.proto",
            "./proto/ica/staking.proto",
            "./proto/query.proto",
        ],
        &["proto"],
    )?;

    Ok(())
}
