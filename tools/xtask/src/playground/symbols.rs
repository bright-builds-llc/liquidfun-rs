//! Fail-closed allocator/`Vec` heuristic over samply sidecars and gzip JSON.

use std::fs;
use std::io::Read;
use std::path::Path;

use flate2::read::GzDecoder;
use serde_json::Value;

use super::PlaygroundError;

const PROFILE_BLOB: &str = "rust.json.gz";
const NEEDLES: [&str; 10] = [
    "alloc::",
    "__rust_alloc",
    "__rdl_alloc",
    "__rg_alloc",
    "alloc::vec::Vec",
    "RawVec",
    "to_vec",
    "GlobalAlloc",
    "core::alloc::",
    "core::clone::",
];

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
pub(super) fn classify_profile_symbols(profile_dir: &Path) -> Result<HeapGate, PlaygroundError> {
    let mut scanned = Vec::new();
    let gzip_path = profile_dir.join(PROFILE_BLOB);
    let primary_sidecar = gzip_path.with_extension("syms.json");
    if primary_sidecar.is_file() {
        collect_symbol_file(&primary_sidecar, &mut scanned)?;
    }
    collect_other_syms_files(profile_dir, &primary_sidecar, &mut scanned)?;
    if gzip_path.is_file() {
        collect_gzip_json_strings(&gzip_path, &mut scanned)?;
    }

    let matched_needles = matched_needles(&scanned);
    if !matched_needles.is_empty() {
        return Ok(HeapGate::Run { matched_needles });
    }
    if scanned.iter().any(|text| is_readable_symbol(text)) {
        return Ok(HeapGate::SkipNoAllocator);
    }
    Ok(HeapGate::SkipNoSymbols)
}

fn collect_other_syms_files(
    profile_dir: &Path,
    primary_sidecar: &Path,
    scanned: &mut Vec<String>,
) -> Result<(), PlaygroundError> {
    let entries = fs::read_dir(profile_dir)
        .map_err(|error| heap_err(format!("failed to read {}: {error}", profile_dir.display())))?;
    let mut extra = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            heap_err(format!("failed to read {}: {error}", profile_dir.display()))
        })?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.contains("syms") || path == primary_sidecar {
            continue;
        }
        extra.push(path);
    }
    extra.sort();
    for path in extra {
        collect_symbol_file(&path, scanned)?;
    }
    Ok(())
}

fn collect_symbol_file(path: &Path, scanned: &mut Vec<String>) -> Result<(), PlaygroundError> {
    let bytes = fs::read(path)
        .map_err(|error| heap_err(format!("failed to read {}: {error}", path.display())))?;
    if let Ok(value) = serde_json::from_slice::<Value>(&bytes) {
        collect_json_strings(&value, scanned);
        return Ok(());
    }
    let Ok(text) = std::str::from_utf8(&bytes) else {
        return Ok(());
    };
    scanned.push(text.to_owned());
    Ok(())
}

fn collect_gzip_json_strings(
    path: &Path,
    scanned: &mut Vec<String>,
) -> Result<(), PlaygroundError> {
    let file = fs::File::open(path)
        .map_err(|error| heap_err(format!("failed to read {}: {error}", path.display())))?;
    let mut decoder = GzDecoder::new(file);
    let mut body = Vec::new();
    if decoder.read_to_end(&mut body).is_err() {
        return Ok(());
    }
    let Ok(value) = serde_json::from_slice::<Value>(&body) else {
        return Ok(());
    };
    if collect_thread_string_array(&value, scanned) {
        return Ok(());
    }
    collect_json_strings(&value, scanned);
    Ok(())
}

fn collect_thread_string_array(value: &Value, scanned: &mut Vec<String>) -> bool {
    let Some(threads) = value.get("threads").and_then(Value::as_array) else {
        return false;
    };
    let mut used = false;
    for thread in threads {
        let Some(array) = thread.get("stringArray").and_then(Value::as_array) else {
            continue;
        };
        used = true;
        for item in array {
            if let Some(text) = item.as_str() {
                scanned.push(text.to_owned());
            }
        }
    }
    used
}

fn collect_json_strings(value: &Value, scanned: &mut Vec<String>) {
    match value {
        Value::String(text) => scanned.push(text.clone()),
        Value::Array(items) => {
            for item in items {
                collect_json_strings(item, scanned);
            }
        }
        Value::Object(map) => {
            for nested in map.values() {
                collect_json_strings(nested, scanned);
            }
        }
        _ => {}
    }
}

fn matched_needles(scanned: &[String]) -> Vec<String> {
    NEEDLES
        .iter()
        .filter(|needle| scanned.iter().any(|text| text.contains(*needle)))
        .map(|needle| (*needle).to_owned())
        .collect()
}

fn is_readable_symbol(text: &str) -> bool {
    !text.is_empty() && !is_hex_address(text)
}

fn is_hex_address(text: &str) -> bool {
    let Some(hex) = text.strip_prefix("0x") else {
        return false;
    };
    !hex.is_empty() && hex.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn heap_err(message: impl Into<String>) -> PlaygroundError {
    PlaygroundError::new("heap", message)
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
