//! Explicit non-ready test input, independent of the live public maturity state.

pub(super) fn non_ready_copy(contents: &str) -> String {
    let mut result = contents
        .lines()
        .filter_map(|line| {
            if line.starts_with("Source candidate:") || line.starts_with("Attestation commit:") {
                return None;
            }
            let maybe_remainder = line
                .strip_prefix("Status: **release-ready**")
                .or_else(|| line.strip_prefix("Status: **not release-ready**"));
            if let Some(remainder) = maybe_remainder {
                return (!remainder.trim().is_empty()).then_some(remainder.trim());
            }
            Some(line)
        })
        .collect::<Vec<_>>()
        .join("\n");
    for (ready, pending) in [
        ("release audit has passed", "release audit is pending"),
        ("release ready", "release pending"),
    ] {
        while let Some(start) = result.to_ascii_lowercase().find(ready) {
            result.replace_range(start..start + ready.len(), pending);
        }
    }
    result.push_str("\nStatus: **not release-ready**\n");
    result
}
