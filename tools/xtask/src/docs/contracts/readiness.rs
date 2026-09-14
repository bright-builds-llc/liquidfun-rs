//! Public maturity remains non-ready unless explicit retained C/A evidence validates.

use std::{fs, path::Path};

use crate::{docs::DocsError, release::attestation};

pub(super) enum Readiness {
    NotReady,
    Attested,
}

impl Readiness {
    pub(super) fn permits(&self, claim: &str) -> bool {
        matches!(self, Self::Attested)
            && matches!(claim, "release ready" | "release audit has passed")
    }
}

pub(super) fn validate(
    root: &Path,
    maybe_attestation_commit: Option<&str>,
) -> Result<Readiness, DocsError> {
    let maybe_candidate = maybe_attestation_commit
        .map(|commit| attestation::validate_committed(root, commit))
        .transpose()
        .map_err(|error| DocsError::new("readiness", error.to_string()))?;
    for document in ["README.md", "COMPATIBILITY.md", "RELEASE.md"] {
        let contents = fs::read_to_string(root.join(document))
            .map_err(|error| DocsError::new("filesystem", error.to_string()))?;
        let required = match (&maybe_candidate, maybe_attestation_commit) {
            (Some(candidate), Some(commit)) => vec![
                "Status: **release-ready**".to_owned(),
                format!("Source candidate: `{candidate}`"),
                format!("Attestation commit: `{commit}`"),
            ],
            _ => vec!["not release-ready".to_owned()],
        };
        for marker in required {
            let present = if maybe_candidate.is_some() {
                let prefix = marker
                    .split_once(':')
                    .expect("readiness markers contain a label")
                    .0;
                let lines = contents
                    .lines()
                    .filter(|line| line.starts_with(&format!("{prefix}:")))
                    .collect::<Vec<_>>();
                lines == [marker.as_str()]
            } else {
                contents.contains(&marker)
            };
            if !present {
                return Err(DocsError::new(
                    "phase12-public-contract",
                    format!("{document} must contain `{marker}`"),
                ));
            }
        }
        if maybe_candidate.is_none()
            && contents
                .lines()
                .any(|line| line == "Status: **release-ready**")
        {
            return Err(DocsError::new(
                "readiness",
                format!("{document} requires explicit valid attestation"),
            ));
        }
    }
    Ok(if maybe_candidate.is_some() {
        Readiness::Attested
    } else {
        Readiness::NotReady
    })
}
