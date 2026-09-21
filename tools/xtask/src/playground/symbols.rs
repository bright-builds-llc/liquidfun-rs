//! Fail-closed allocator/`Vec` heuristic over samply sidecars and gzip JSON.

use std::path::Path;

use super::PlaygroundError;

/// Outcome of the sidecar-first allocator/`Vec` needle scan.
#[allow(dead_code)]
pub(super) enum HeapGate {
    /// At least one D-12 needle was present in scanned symbol strings.
    Run { matched_needles: Vec<String> },
    /// Symbol strings were readable, but none matched allocator/`Vec` needles.
    SkipNoAllocator,
    /// Gzip/sidecar evidence could not be read as symbols.
    SkipNoSymbols,
}

/// Classifies a profile stamp for the private dhat heap gate.
///
/// # Errors
///
/// Returns a closed error when a readable sidecar or gzip file cannot be opened.
#[allow(dead_code)]
pub(super) fn classify_profile_symbols(_profile_dir: &Path) -> Result<HeapGate, PlaygroundError> {
    todo!("classify_profile_symbols")
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    use flate2::Compression;
    use flate2::write::GzEncoder;

    use super::{HeapGate, classify_profile_symbols};

    static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    fn fixture_dir(label: &str) -> PathBuf {
        let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("target/xtask-test-fixtures")
            .join(format!(
                "playground-symbols-{}-{id}-{label}",
                std::process::id()
            ));
        fs::create_dir_all(&dir).expect("fixture dir should be created");
        dir
    }

    fn write_gzip(path: &Path, body: &[u8]) {
        let file = fs::File::create(path).expect("gzip file should be created");
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(body).expect("gzip body should write");
        encoder.finish().expect("gzip should finish");
    }

    fn assert_run_includes_needle(gate: HeapGate, needle: &str) {
        let HeapGate::Run { matched_needles } = gate else {
            panic!("expected HeapGate::Run, got a skip variant");
        };
        assert!(
            matched_needles.iter().any(|matched| matched == needle),
            "matched_needles {matched_needles:?} should include {needle}"
        );
    }

    #[test]
    fn sidecar_vec_needle_runs_even_when_gzip_is_raw_bytes() {
        // Arrange
        let dir = fixture_dir("sidecar-first");
        fs::write(dir.join("rust.json.gz"), b"not-gzip-or-json")
            .expect("raw gzip placeholder should write");
        fs::write(
            dir.join("rust.json.syms.json"),
            r#"{"string_table":["alloc::vec::Vec"]}"#,
        )
        .expect("sidecar should write");

        // Act
        let gate = classify_profile_symbols(&dir).expect("sidecar scan should succeed");

        // Assert
        let HeapGate::Run { matched_needles } = gate else {
            panic!("expected HeapGate::Run from sidecar Vec needle");
        };
        assert!(
            matched_needles
                .iter()
                .any(|needle| needle == "alloc::" || needle == "alloc::vec::Vec"),
            "matched_needles {matched_needles:?} should include alloc:: or alloc::vec::Vec"
        );
    }

    #[test]
    fn readable_gzip_without_needles_skips_no_allocator() {
        // Arrange
        let dir = fixture_dir("no-allocator");
        write_gzip(
            &dir.join("rust.json.gz"),
            br#"{"threads":[{"stringArray":["liquidfun::particle::proxy::from_view"]}]}"#,
        );

        // Act
        let gate = classify_profile_symbols(&dir).expect("gzip scan should succeed");

        // Assert
        assert!(matches!(gate, HeapGate::SkipNoAllocator));
    }

    #[test]
    fn gzip_core_clone_needle_runs() {
        // Arrange
        let dir = fixture_dir("core-clone");
        write_gzip(
            &dir.join("rust.json.gz"),
            br#"{"threads":[{"stringArray":["core::clone::Clone"]}]}"#,
        );

        // Act
        let gate = classify_profile_symbols(&dir).expect("gzip scan should succeed");

        // Assert
        assert_run_includes_needle(gate, "core::clone::");
    }

    #[test]
    fn uncompressed_fake_samply_bytes_skip_no_symbols() {
        // Arrange
        let dir = fixture_dir("fake-bytes");
        fs::write(dir.join("rust.json.gz"), b"fake-samply-json-gz")
            .expect("fake samply bytes should write");

        // Act
        let gate = classify_profile_symbols(&dir).expect("inconclusive scan should succeed");

        // Assert
        assert!(matches!(gate, HeapGate::SkipNoSymbols));
    }

    #[test]
    fn gzip_hex_only_addresses_skip_no_symbols() {
        // Arrange
        let dir = fixture_dir("hex-only");
        write_gzip(&dir.join("rust.json.gz"), br#"["0x1000","0x2000"]"#);

        // Act
        let gate = classify_profile_symbols(&dir).expect("address-only scan should succeed");

        // Assert
        assert!(matches!(gate, HeapGate::SkipNoSymbols));
    }

    #[test]
    fn bare_clone_substring_does_not_run() {
        // Arrange
        let dir = fixture_dir("bare-clone");
        write_gzip(
            &dir.join("rust.json.gz"),
            br#"{"threads":[{"stringArray":["MycloneHelper"]}]}"#,
        );

        // Act
        let gate = classify_profile_symbols(&dir).expect("bare clone scan should succeed");

        // Assert
        assert!(
            !matches!(gate, HeapGate::Run { .. }),
            "bare substring clone must not produce HeapGate::Run"
        );
    }
}
