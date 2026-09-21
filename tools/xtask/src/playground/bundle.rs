//! Copy-only same-HEAD Dam Break audit bundle.

mod ops;

use std::fs;
use std::path::Path;

use super::PlaygroundError;

#[cfg(test)]
pub(super) use ops::copy_audit_bundle;
pub(super) use ops::run;

fn copy_file(from: &Path, to: &Path) -> Result<(), PlaygroundError> {
    fs::copy(from, to).map_err(|error| {
        PlaygroundError::new(
            "bundle",
            format!(
                "failed to copy {} to {}: {error}",
                from.display(),
                to.display()
            ),
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::copy_audit_bundle;

    static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    const PAIR_STAMP: &str = "2001-09-09T01-46-40Z";
    const PROFILE_STAMP: &str = "2001-09-09T01-46-41Z";
    const HEAD_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HEAD_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const PAIR_MD: &str = "# Unreviewed pair\n";
    const PROFILE_BLOB: &[u8] = b"fake-samply-json-gz";
    const SYMS_BLOB: &[u8] = b"{\"syms\":true}";

    struct SourceBytes {
        pair_json: Vec<u8>,
        pair_md: Vec<u8>,
        profile_blob: Vec<u8>,
        syms: Vec<u8>,
    }

    fn fixture_root(label: &str) -> PathBuf {
        let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("target/xtask-test-fixtures")
            .join(format!(
                "playground-bundle-{}-{id}-{label}",
                std::process::id()
            ))
    }

    fn pair_json(git_head: &str) -> String {
        format!(
            r#"{{"kind":"unprofiled_pair","timing_authority":"unprofiled_wall_clock","git_head":"{git_head}","rust_over_cpp_ratio":5.2}}"#
        )
    }

    fn profile_identity_json(git_head: &str) -> String {
        format!(r#"{{"kind":"samply_cpu","not_timing_authority":true,"git_head":"{git_head}"}}"#)
    }

    fn write_pair_stamp(root: &Path, git_head: &str) -> SourceBytes {
        let pair_dir = root.join("target/dam-break-perf").join(PAIR_STAMP);
        fs::create_dir_all(&pair_dir).expect("pair stamp should be created");
        let pair_json = pair_json(git_head).into_bytes();
        let pair_md = PAIR_MD.as_bytes().to_vec();
        fs::write(pair_dir.join("pair.json"), &pair_json).expect("pair.json should be written");
        fs::write(pair_dir.join("pair.md"), &pair_md).expect("pair.md should be written");
        SourceBytes {
            pair_json,
            pair_md,
            profile_blob: PROFILE_BLOB.to_vec(),
            syms: SYMS_BLOB.to_vec(),
        }
    }

    fn write_profile_stamp(root: &Path, git_head: &str, write_blob: bool) {
        let profile_dir = root.join("target/dam-break-perf").join(PROFILE_STAMP);
        fs::create_dir_all(&profile_dir).expect("profile stamp should be created");
        fs::write(
            profile_dir.join("profile-identity.json"),
            profile_identity_json(git_head),
        )
        .expect("profile-identity.json should be written");
        if write_blob {
            fs::write(profile_dir.join("rust.json.gz"), PROFILE_BLOB)
                .expect("rust.json.gz should be written");
            fs::write(profile_dir.join("rust.json.syms.json"), SYMS_BLOB)
                .expect("syms sidecar should be written");
        }
    }

    fn original_source_bytes(root: &Path) -> SourceBytes {
        let pair_dir = root.join("target/dam-break-perf").join(PAIR_STAMP);
        let profile_dir = root.join("target/dam-break-perf").join(PROFILE_STAMP);
        SourceBytes {
            pair_json: fs::read(pair_dir.join("pair.json")).expect("source pair.json"),
            pair_md: fs::read(pair_dir.join("pair.md")).expect("source pair.md"),
            profile_blob: fs::read(profile_dir.join("rust.json.gz")).expect("source rust.json.gz"),
            syms: fs::read(profile_dir.join("rust.json.syms.json")).expect("source syms"),
        }
    }

    fn assert_sources_untouched(root: &Path, original: &SourceBytes) {
        let current = original_source_bytes(root);
        assert_eq!(current.pair_json, original.pair_json);
        assert_eq!(current.pair_md, original.pair_md);
        assert_eq!(current.profile_blob, original.profile_blob);
        assert_eq!(current.syms, original.syms);
        assert!(
            root.join("target/dam-break-perf")
                .join(PAIR_STAMP)
                .join("pair.json")
                .is_file(),
            "source pair.json must still exist"
        );
        assert!(
            root.join("target/dam-break-perf")
                .join(PROFILE_STAMP)
                .join("rust.json.gz")
                .is_file(),
            "source rust.json.gz must still exist"
        );
    }

    fn bundle_identity_dirs(root: &Path) -> Vec<PathBuf> {
        let evidence = root.join("target/dam-break-perf");
        let Ok(entries) = fs::read_dir(&evidence) else {
            return Vec::new();
        };
        let mut dirs = Vec::new();
        for entry in entries {
            let Ok(entry) = entry else {
                continue;
            };
            let path = entry.path();
            if path.is_dir() && path.join("audit-bundle-identity.json").is_file() {
                dirs.push(path);
            }
        }
        dirs
    }

    #[test]
    fn copy_audit_bundle_copies_pair_profile_and_syms_without_moving_sources() {
        // Arrange
        let root = fixture_root("copy");
        let original = write_pair_stamp(&root, HEAD_A);
        write_profile_stamp(&root, HEAD_A, true);

        // Act
        let dest = copy_audit_bundle(
            &root,
            HEAD_A,
            Some(PAIR_STAMP),
            Some(PROFILE_STAMP),
            1_000_000_002,
            None,
        )
        .expect("same-HEAD sources should copy into a new stamp");

        // Assert
        let copied_pair = fs::read(dest.join("pair.json")).expect("copied pair.json");
        let copied_md = fs::read(dest.join("pair.md")).expect("copied pair.md");
        let copied_blob = fs::read(dest.join("rust.json.gz")).expect("copied rust.json.gz");
        let copied_syms = fs::read(dest.join("rust.json.syms.json")).expect("copied syms");
        assert_eq!(copied_pair, original.pair_json);
        assert_eq!(copied_md, original.pair_md);
        assert_eq!(copied_blob, original.profile_blob);
        assert_eq!(copied_syms, original.syms);
        assert_sources_untouched(&root, &original);
        assert_ne!(
            dest.file_name().and_then(|name| name.to_str()),
            Some(PAIR_STAMP)
        );
        assert_ne!(
            dest.file_name().and_then(|name| name.to_str()),
            Some(PROFILE_STAMP)
        );
    }

    #[test]
    fn copy_audit_bundle_rejects_mismatched_git_heads() {
        // Arrange
        let root = fixture_root("mismatch");
        let original = write_pair_stamp(&root, HEAD_A);
        write_profile_stamp(&root, HEAD_B, true);

        // Act
        let result = copy_audit_bundle(
            &root,
            HEAD_A,
            Some(PAIR_STAMP),
            Some(PROFILE_STAMP),
            1_000_000_002,
            None,
        );

        // Assert
        assert!(result.is_err(), "HEAD mismatch must fail closed");
        assert!(
            bundle_identity_dirs(&root).is_empty(),
            "mismatch must not write audit-bundle-identity.json"
        );
        assert_sources_untouched(&root, &original);
    }

    #[test]
    fn copy_audit_bundle_rejects_missing_rust_json_gz() {
        // Arrange
        let root = fixture_root("missing-gz");
        write_pair_stamp(&root, HEAD_A);
        write_profile_stamp(&root, HEAD_A, false);
        let pair_json = fs::read(
            root.join("target/dam-break-perf")
                .join(PAIR_STAMP)
                .join("pair.json"),
        )
        .expect("source pair.json");
        let pair_md = fs::read(
            root.join("target/dam-break-perf")
                .join(PAIR_STAMP)
                .join("pair.md"),
        )
        .expect("source pair.md");

        // Act
        let result = copy_audit_bundle(
            &root,
            HEAD_A,
            Some(PAIR_STAMP),
            Some(PROFILE_STAMP),
            1_000_000_002,
            None,
        );

        // Assert
        assert!(result.is_err(), "missing rust.json.gz must fail closed");
        assert!(bundle_identity_dirs(&root).is_empty());
        assert_eq!(
            fs::read(
                root.join("target/dam-break-perf")
                    .join(PAIR_STAMP)
                    .join("pair.json")
            )
            .expect("pair.json stays"),
            pair_json
        );
        assert_eq!(
            fs::read(
                root.join("target/dam-break-perf")
                    .join(PAIR_STAMP)
                    .join("pair.md")
            )
            .expect("pair.md stays"),
            pair_md
        );
        assert!(
            !root
                .join("target/dam-break-perf")
                .join(PROFILE_STAMP)
                .join("rust.json.gz")
                .exists()
        );
    }

    #[test]
    fn copy_audit_bundle_identity_marks_profile_not_timing_authority() {
        // Arrange
        let root = fixture_root("identity");
        write_pair_stamp(&root, HEAD_A);
        write_profile_stamp(&root, HEAD_A, true);

        // Act
        let dest = copy_audit_bundle(
            &root,
            HEAD_A,
            Some(PAIR_STAMP),
            Some(PROFILE_STAMP),
            1_000_000_002,
            None,
        )
        .expect("bundle identity should be written");

        // Assert
        let identity: serde_json::Value = serde_json::from_slice(
            &fs::read(dest.join("audit-bundle-identity.json")).expect("identity json"),
        )
        .expect("identity must parse");
        assert_eq!(identity["kind"], "audit_bundle");
        assert_eq!(identity["not_timing_authority"], true);
        assert_eq!(identity["timing_authority"], "unprofiled_wall_clock");
        assert_eq!(identity["profile_blob"], "rust.json.gz");
        let copied = identity["copied"]
            .as_array()
            .expect("copied should be an array");
        let names: Vec<&str> = copied
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect();
        assert!(names.contains(&"pair.json"));
        assert!(names.contains(&"pair.md"));
        assert!(names.contains(&"rust.json.gz"));
    }

    #[test]
    fn copy_audit_bundle_keeps_pair_unprofiled_timing_authority() {
        // Arrange
        let root = fixture_root("pair-authority");
        write_pair_stamp(&root, HEAD_A);
        write_profile_stamp(&root, HEAD_A, true);

        // Act
        let dest = copy_audit_bundle(
            &root,
            HEAD_A,
            Some(PAIR_STAMP),
            Some(PROFILE_STAMP),
            1_000_000_002,
            None,
        )
        .expect("copied pair.json should keep wall-clock authority");

        // Assert
        let pair: serde_json::Value =
            serde_json::from_slice(&fs::read(dest.join("pair.json")).expect("copied pair.json"))
                .expect("copied pair.json must parse");
        assert_eq!(pair["timing_authority"], "unprofiled_wall_clock");
        assert!(pair.get("samply").is_none());
    }
}
