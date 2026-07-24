use std::fs::OpenOptions;
use std::io;
use std::path::Path;

const OBSERVATION_DIRECTORY_ENV: &str = "LIQUIDFUN_DIFFERENTIAL_LEAF_DIRECTORY";

pub fn observe(leaves: &[&str]) -> io::Result<()> {
    let Some(directory) = std::env::var_os(OBSERVATION_DIRECTORY_ENV) else {
        return Ok(());
    };
    let directory = Path::new(&directory);
    for leaf in leaves {
        if leaf.is_empty()
            || !leaf.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || b".-".contains(&byte)
            })
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "differential coverage leaf ID is invalid",
            ));
        }
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join(leaf))
        {
            Ok(_marker) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}
