//! Exclusive UTC evidence stamps for playground Dam Break artifacts.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use super::PlaygroundError;

const EVIDENCE_RELATIVE_DIR: &str = "target/dam-break-perf";
const MAX_STAMP_ATTEMPTS: u8 = 8;
const SECONDS_PER_DAY: u64 = 86_400;
const SECONDS_PER_HOUR: u64 = 3_600;
const SECONDS_PER_MINUTE: u64 = 60;

/// Formats Unix seconds as a filename-safe UTC stamp `YYYY-MM-DDTHH-MM-SSZ`.
pub(crate) fn format_utc_stamp(unix_seconds: u64) -> String {
    let unix_days = unix_seconds / SECONDS_PER_DAY;
    let seconds_of_day = unix_seconds % SECONDS_PER_DAY;
    let hour = seconds_of_day / SECONDS_PER_HOUR;
    let minute = (seconds_of_day % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;
    let second = seconds_of_day % SECONDS_PER_MINUTE;
    let (year, month, day) = civil_date_from_unix_days(unix_days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}-{minute:02}-{second:02}Z")
}

/// Creates an exclusive `target/dam-break-perf/<utc-stamp>/` directory.
///
/// # Errors
///
/// Returns a closed error when the evidence parent cannot be created or every
/// stamp candidate in the retry window already exists.
pub(crate) fn mint_exclusive_stamp(
    repository_root: &Path,
    unix_seconds: u64,
) -> Result<PathBuf, PlaygroundError> {
    let evidence_root = repository_root.join(EVIDENCE_RELATIVE_DIR);
    fs::create_dir_all(&evidence_root).map_err(|error| {
        PlaygroundError::new(
            "stamp",
            format!(
                "failed to create evidence directory {}: {error}",
                evidence_root.display()
            ),
        )
    })?;

    let mut candidate_seconds = unix_seconds;
    let mut remaining_attempts = MAX_STAMP_ATTEMPTS;
    while remaining_attempts > 0 {
        remaining_attempts -= 1;
        let stamp = format_utc_stamp(candidate_seconds);
        let stamp_path = evidence_root.join(&stamp);
        match fs::create_dir(&stamp_path) {
            Ok(()) => return Ok(stamp_path),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                let Some(next_seconds) = candidate_seconds.checked_add(1) else {
                    break;
                };
                candidate_seconds = next_seconds;
            }
            Err(error) => {
                return Err(PlaygroundError::new(
                    "stamp",
                    format!("failed to create {}: {error}", stamp_path.display()),
                ));
            }
        }
    }

    Err(PlaygroundError::new(
        "stamp",
        format!(
            "stamp already exists for {MAX_STAMP_ATTEMPTS} consecutive seconds starting at {}",
            format_utc_stamp(unix_seconds)
        ),
    ))
}

/// Converts days since 1970-01-01 into a Gregorian civil date.
///
/// Algorithm from Howard Hinnant's public-domain `civil_from_days`.
fn civil_date_from_unix_days(unix_days: u64) -> (i32, u32, u32) {
    let Ok(unix_days_i64) = i64::try_from(unix_days) else {
        return (1970, 1, 1);
    };
    let shifted_days = unix_days_i64.saturating_add(719_468);
    let era = if shifted_days >= 0 {
        shifted_days
    } else {
        shifted_days.saturating_sub(146_096)
    } / 146_097;
    let Ok(day_of_era) = u32::try_from(shifted_days - era * 146_097) else {
        return (1970, 1, 1);
    };
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = if month_prime < 10 {
        month_prime + 3
    } else {
        month_prime - 9
    };
    let mut year = i64::from(year_of_era) + era * 400;
    if month <= 2 {
        year += 1;
    }
    let Ok(year) = i32::try_from(year) else {
        return (1970, 1, 1);
    };
    (year, month, day)
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
