use std::path::{Path, PathBuf};

use super::PlaygroundError;

/// Formats Unix seconds as a filename-safe UTC stamp `YYYY-MM-DDTHH-MM-SSZ`.
pub(crate) fn format_utc_stamp(_unix_seconds: u64) -> String {
    String::new()
}

/// Creates an exclusive `target/dam-break-perf/<utc-stamp>/` directory.
///
/// # Errors
///
/// Returns a closed error when the evidence parent cannot be created or every
/// stamp candidate in the retry window already exists.
pub(crate) fn mint_exclusive_stamp(
    _repository_root: &Path,
    _unix_seconds: u64,
) -> Result<PathBuf, PlaygroundError> {
    Err(PlaygroundError::new("unimplemented", "stamp mint stub"))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::{format_utc_stamp, mint_exclusive_stamp};

    static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    const OCCUPIED_STAMPS: [&str; 8] = [
        "2001-09-09T01-46-40Z",
        "2001-09-09T01-46-41Z",
        "2001-09-09T01-46-42Z",
        "2001-09-09T01-46-43Z",
        "2001-09-09T01-46-44Z",
        "2001-09-09T01-46-45Z",
        "2001-09-09T01-46-46Z",
        "2001-09-09T01-46-47Z",
    ];

    fn fixture_root(label: &str) -> PathBuf {
        let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("target/xtask-test-fixtures")
            .join(format!(
                "playground-stamp-{}-{id}-{label}",
                std::process::id()
            ))
    }

    #[test]
    fn format_utc_stamp_formats_unix_epoch() {
        // Arrange / Act
        let stamp = format_utc_stamp(0);

        // Assert
        assert_eq!(stamp, "1970-01-01T00-00-00Z");
    }

    #[test]
    fn format_utc_stamp_formats_one_billion_seconds() {
        // Arrange / Act
        let stamp = format_utc_stamp(1_000_000_000);

        // Assert
        assert_eq!(stamp, "2001-09-09T01-46-40Z");
    }

    #[test]
    fn format_utc_stamp_is_filename_safe() {
        // Arrange
        let samples = [0_u64, 1, 86_401, 1_000_000_000];

        // Act / Assert
        for unix_seconds in samples {
            let stamp = format_utc_stamp(unix_seconds);
            assert!(
                !stamp.contains(':') && !stamp.contains('/') && !stamp.contains(' '),
                "stamp `{stamp}` must not contain `:`, `/`, or spaces"
            );
            assert!(
                stamp.len() == 20
                    && stamp
                        .chars()
                        .all(|ch| ch.is_ascii_digit() || ch == '-' || ch == 'T' || ch == 'Z')
                    && stamp.as_bytes()[4] == b'-'
                    && stamp.as_bytes()[7] == b'-'
                    && stamp.as_bytes()[10] == b'T'
                    && stamp.as_bytes()[13] == b'-'
                    && stamp.as_bytes()[16] == b'-'
                    && stamp.ends_with('Z'),
                "stamp `{stamp}` must match YYYY-MM-DDTHH-MM-SSZ"
            );
        }
    }

    #[test]
    fn mint_exclusive_stamp_creates_dated_evidence_directory() {
        // Arrange
        let root = fixture_root("create");
        fs::create_dir_all(&root).expect("fixture root should be created");

        // Act
        let path = mint_exclusive_stamp(&root, 1_000_000_000)
            .expect("first stamp should be created exclusively");

        // Assert
        assert_eq!(
            path,
            root.join("target/dam-break-perf/2001-09-09T01-46-40Z")
        );
        assert!(path.is_dir());
    }

    #[test]
    fn mint_exclusive_stamp_bumps_one_second_when_stamp_exists() {
        // Arrange
        let root = fixture_root("collision");
        let evidence_root = root.join("target/dam-break-perf");
        fs::create_dir_all(&evidence_root).expect("evidence parent should be created");
        let existing = evidence_root.join("2001-09-09T01-46-40Z");
        fs::create_dir(&existing).expect("occupied stamp should be created with exclusive mkdir");
        let marker = existing.join("marker.txt");
        fs::write(&marker, b"keep-me").expect("marker should be written");

        // Act
        let path = mint_exclusive_stamp(&root, 1_000_000_000)
            .expect("colliding stamp should bump one second");

        // Assert
        assert_eq!(
            path,
            root.join("target/dam-break-perf/2001-09-09T01-46-41Z")
        );
        assert!(path.is_dir());
        let marker_bytes = fs::read(&marker).expect("original marker should still be readable");
        assert_eq!(marker_bytes, b"keep-me");
    }

    #[test]
    fn mint_exclusive_stamp_fails_closed_after_eight_existing_stamps() {
        // Arrange
        let root = fixture_root("exhausted");
        let evidence_root = root.join("target/dam-break-perf");
        fs::create_dir_all(&evidence_root).expect("evidence parent should be created");
        for stamp in OCCUPIED_STAMPS {
            let occupied = evidence_root.join(stamp);
            fs::create_dir(&occupied).expect("occupied stamp should exist");
            fs::write(occupied.join("keep"), b"x").expect("occupied stamp should keep a marker");
        }

        // Act
        let result = mint_exclusive_stamp(&root, 1_000_000_000);

        // Assert
        let Err(error) = result else {
            panic!("eight consecutive stamps should fail closed");
        };
        let display = error.to_string();
        assert!(
            display.contains("stamp"),
            "error `{display}` should mention stamp"
        );
        assert!(
            display.contains("already exists") || display.contains("exists"),
            "error `{display}` should mention exists"
        );
        assert!(!evidence_root.join("2001-09-09T01-46-48Z").exists());
        for stamp in OCCUPIED_STAMPS {
            let marker = evidence_root.join(stamp).join("keep");
            let marker_bytes = fs::read(&marker).expect("occupied markers must stay unchanged");
            assert_eq!(marker_bytes, b"x");
        }
    }
}
