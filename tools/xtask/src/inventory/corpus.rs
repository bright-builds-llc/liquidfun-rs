//! Strict parsing boundary for the semantic upstream corpus authority.

#[path = "corpus/model.rs"]
pub(crate) mod model;

use serde::Deserialize;

use model::RawCorpusManifest;
pub(crate) use model::{CorpusError, CorpusErrorKind, CorpusManifest};

pub(crate) const MAX_CORPUS_BYTES: usize = 4 * 1024 * 1024;
pub(crate) const MAX_JSON_DEPTH: usize = 32;

/// Parses untrusted corpus JSON into the checked semantic authority model.
pub(crate) fn parse_manifest(
    bytes: &[u8],
    expected_revision: &str,
) -> Result<CorpusManifest, CorpusError> {
    if bytes.len() > MAX_CORPUS_BYTES {
        return Err(CorpusError::new(CorpusErrorKind::InputLimit));
    }
    require_bounded_json_depth(bytes)?;
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let raw = RawCorpusManifest::deserialize(&mut deserializer)
        .map_err(|_| CorpusError::new(CorpusErrorKind::Schema))?;
    deserializer
        .end()
        .map_err(|_| CorpusError::new(CorpusErrorKind::Schema))?;
    CorpusManifest::from_raw(raw, expected_revision)
}

fn require_bounded_json_depth(bytes: &[u8]) -> Result<(), CorpusError> {
    let mut depth = 0_usize;
    let mut in_string = false;
    let mut escaped = false;

    for byte in bytes {
        if in_string {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                in_string = false;
            }
            continue;
        }

        match byte {
            b'"' => in_string = true,
            b'{' | b'[' => {
                depth = depth
                    .checked_add(1)
                    .ok_or_else(|| CorpusError::new(CorpusErrorKind::DepthLimit))?;
                if depth > MAX_JSON_DEPTH {
                    return Err(CorpusError::new(CorpusErrorKind::DepthLimit));
                }
            }
            b'}' | b']' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    Ok(())
}
