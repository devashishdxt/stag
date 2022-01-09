use std::{
    error::Error,
    fs::{read_dir, DirEntry},
    path::PathBuf,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut files = Vec::new();

    let paths = read_dir("./proto")?;

    for path in paths {
        files.extend(get_files(path?)?);
    }

    tonic_build::configure()
        .build_server(false)
        .extern_path(
            ".cosmos.auth.v1beta1",
            "::cosmos_sdk_proto::cosmos::auth::v1beta1",
        )
        .extern_path(".cosmos.base", "::cosmos_sdk_proto::cosmos::base")
        .extern_path(
            ".cosmos.bank.v1beta1",
            "::cosmos_sdk_proto::cosmos::bank::v1beta1",
        )
        .extern_path(
            ".cosmos.staking.v1beta1",
            "::cosmos_sdk_proto::cosmos::staking::v1beta1",
        )
        .extern_path(".ibc.core", "::cosmos_sdk_proto::ibc::core")
        .compile(&files, &["proto"])?;

    Ok(())
}

fn get_files(path: DirEntry) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    if path.file_type()?.is_file() {
        return Ok(vec![path.path()]);
    }

    let paths = read_dir(path.path())?;
    let mut files = Vec::new();

    for path in paths {
        files.extend(get_files(path?)?);
    }

    Ok(files)
}
