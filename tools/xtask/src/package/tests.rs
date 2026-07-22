use std::error::Error;
use std::fs;
use std::io::{self, Cursor};

use flate2::Compression;
use flate2::write::GzEncoder;

use super::{TemporaryDirectory, extract_archive, inspect_archive, read_archive};

type TestResult = Result<(), Box<dyn Error>>;

#[test]
fn extraction_uses_the_exact_archive_bytes_that_passed_inspection() -> TestResult {
    // Arrange
    let temporary = TemporaryDirectory::create()?;
    let archive_path = temporary.path.join("package.crate");
    fs::write(&archive_path, archive_bytes("original")?)?;
    let trusted_bytes = read_archive(&archive_path)?;
    inspect_archive(&trusted_bytes, &archive_path, "liquidfun-0.0.0")?;

    // Act
    fs::write(&archive_path, archive_bytes("replacement")?)?;
    let destination = temporary.path.join("unpacked");
    fs::create_dir(&destination)?;
    extract_archive(&trusted_bytes, &archive_path, &destination)?;

    // Assert
    let manifest = fs::read_to_string(destination.join("liquidfun-0.0.0/Cargo.toml"))?;
    assert!(manifest.contains("# original"));
    assert!(!manifest.contains("# replacement"));
    Ok(())
}

fn archive_bytes(marker: &str) -> io::Result<Vec<u8>> {
    let encoder = GzEncoder::new(Vec::new(), Compression::default());
    let mut archive = tar::Builder::new(encoder);
    append_file(
        &mut archive,
        "liquidfun-0.0.0/Cargo.toml",
        format!("[package]\nname = \"liquidfun\"\nversion = \"0.0.0\"\n# {marker}\n").as_bytes(),
    )?;
    append_file(
        &mut archive,
        "liquidfun-0.0.0/LICENSE",
        b"fixture license\n",
    )?;
    archive.finish()?;
    archive.into_inner()?.finish()
}

fn append_file<W: io::Write>(
    archive: &mut tar::Builder<W>,
    path: &str,
    contents: &[u8],
) -> io::Result<()> {
    let mut header = tar::Header::new_gnu();
    header.set_size(u64::try_from(contents.len()).map_err(io::Error::other)?);
    header.set_mode(0o644);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_entry_type(tar::EntryType::Regular);
    header.set_cksum();
    archive.append_data(&mut header, path, Cursor::new(contents))
}
