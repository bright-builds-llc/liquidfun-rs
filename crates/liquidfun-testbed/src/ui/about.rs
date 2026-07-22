//! Safe, bounded source, license, maintainer, and build provenance chrome.

#![allow(
    missing_docs,
    reason = "bounded provenance fields and accessors share their visible UI labels"
)]

use super::UNAVAILABLE;

pub const SOURCE_URL: &str = "https://github.com/bright-builds-llc/liquidfun-rs";
pub const LICENSE_URL: &str = "https://github.com/bright-builds-llc/liquidfun-rs/blob/main/LICENSE";
pub const UPSTREAM_NOTICES_URL: &str =
    "https://github.com/bright-builds-llc/liquidfun-rs/blob/main/UPSTREAM.md";
pub const OPENLINKS_URL: &str = "https://openlinks.us/";

const COMMIT_URL_PREFIX: &str = "https://github.com/bright-builds-llc/liquidfun-rs/commit/";
const MAXIMUM_VISIBLE_PROVENANCE_BYTES: usize = 192;

/// Untrusted optional build metadata entering the visual boundary.
#[derive(Debug, Clone, Copy, Default)]
pub struct ProvenanceInput<'a> {
    pub version: Option<&'a str>,
    pub commit: Option<&'a str>,
    pub profile: Option<&'a str>,
    pub target: Option<&'a str>,
    pub rust_toolchain: Option<&'a str>,
    pub protocol_version: Option<&'a str>,
    pub adapter_version: Option<&'a str>,
    pub run_identity: Option<&'a str>,
    pub oracle_revision: Option<&'a str>,
    pub oracle_compiler: Option<&'a str>,
    pub oracle_preset: Option<&'a str>,
    pub evidence_tier: Option<&'a str>,
}

/// Safe platform intent with a visible URL-copy fallback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalLinkAction {
    /// Ask the platform to open the allowlisted URL; retain the same URL for copying.
    OpenOrCopy { url: Box<str> },
}

/// One fixed or validated HTTPS disclosure link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeExternalLink {
    label: &'static str,
    url: Box<str>,
}

impl SafeExternalLink {
    fn fixed(label: &'static str, url: &'static str) -> Self {
        debug_assert!(url.starts_with("https://"));
        Self {
            label,
            url: url.into(),
        }
    }

    fn commit(commit: &str) -> Option<Self> {
        valid_commit(commit).then(|| Self {
            label: "View commit",
            url: format!("{COMMIT_URL_PREFIX}{commit}").into_boxed_str(),
        })
    }

    /// Returns the visible accessible link label.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        self.label
    }

    /// Returns the validated HTTPS URL.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Every external action has a visible copy fallback.
    #[must_use]
    pub const fn copyable_fallback(&self) -> bool {
        true
    }

    /// Returns the safe platform effect request.
    #[must_use]
    pub fn action(&self) -> ExternalLinkAction {
        ExternalLinkAction::OpenOrCopy {
            url: self.url.clone(),
        }
    }
}

/// Complete bounded About and provenance presentation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AboutPanel {
    version_label: Box<str>,
    commit_label: Box<str>,
    commit_url: Box<str>,
    profile: Box<str>,
    target: Box<str>,
    rust_toolchain: Box<str>,
    protocol_version: Box<str>,
    adapter_version: Box<str>,
    run_identity: Box<str>,
    oracle_identity: Box<str>,
    evidence_tier: Box<str>,
    links: Vec<SafeExternalLink>,
}

/// Builds bounded product chrome with literal fallbacks and allowlisted links.
#[must_use]
pub fn build_about_panel(input: ProvenanceInput<'_>) -> AboutPanel {
    let maybe_version = sanitized(input.version);
    let maybe_commit = input.commit.filter(|value| valid_commit(value));
    let version_label = maybe_version.map_or_else(
        || "Version Unavailable".to_owned(),
        |value| format!("Version {value}"),
    );
    let commit_label = maybe_commit.map_or_else(
        || "Commit Unavailable".to_owned(),
        |value| format!("Commit {}", &value[..value.len().min(12)]),
    );
    let maybe_commit_link = maybe_commit.and_then(SafeExternalLink::commit);
    let commit_url = maybe_commit_link
        .as_ref()
        .map_or_else(|| UNAVAILABLE.to_owned(), |link| link.url().to_owned());
    let oracle_identity = oracle_summary(&input);
    let mut links = vec![
        SafeExternalLink::fixed("View source", SOURCE_URL),
        SafeExternalLink::fixed("MIT license", LICENSE_URL),
        SafeExternalLink::fixed(
            "LiquidFun/Box2D provenance and notices",
            UPSTREAM_NOTICES_URL,
        ),
        SafeExternalLink::fixed("OpenLinks", OPENLINKS_URL),
    ];
    if let Some(commit_link) = maybe_commit_link {
        links.push(commit_link);
    }
    AboutPanel {
        version_label: version_label.into_boxed_str(),
        commit_label: commit_label.into_boxed_str(),
        commit_url: commit_url.into_boxed_str(),
        profile: fallback(input.profile),
        target: fallback(input.target),
        rust_toolchain: fallback(input.rust_toolchain),
        protocol_version: fallback(input.protocol_version),
        adapter_version: fallback(input.adapter_version),
        run_identity: fallback(input.run_identity),
        oracle_identity: oracle_identity.into_boxed_str(),
        evidence_tier: fallback(input.evidence_tier),
        links,
    }
}

impl AboutPanel {
    #[must_use]
    pub const fn project_name(&self) -> &'static str {
        "liquidfun-rs"
    }

    #[must_use]
    pub const fn maintainer(&self) -> &'static str {
        "By Peter Ryszkiewicz"
    }

    #[must_use]
    pub const fn license_summary(&self) -> &'static str {
        "MIT-licensed open-source Rust project"
    }

    #[must_use]
    pub const fn upstream_summary(&self) -> &'static str {
        "LiquidFun/Box2D provenance and notices"
    }

    #[must_use]
    pub const fn source_url(&self) -> &'static str {
        SOURCE_URL
    }

    #[must_use]
    pub const fn openlinks_url(&self) -> &'static str {
        OPENLINKS_URL
    }

    #[must_use]
    pub fn version_label(&self) -> &str {
        &self.version_label
    }

    #[must_use]
    pub fn commit_label(&self) -> &str {
        &self.commit_label
    }

    #[must_use]
    pub fn commit_url(&self) -> &str {
        &self.commit_url
    }

    #[must_use]
    pub fn profile(&self) -> &str {
        &self.profile
    }

    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }

    #[must_use]
    pub fn rust_toolchain(&self) -> &str {
        &self.rust_toolchain
    }

    #[must_use]
    pub fn protocol_version(&self) -> &str {
        &self.protocol_version
    }

    #[must_use]
    pub fn adapter_version(&self) -> &str {
        &self.adapter_version
    }

    #[must_use]
    pub fn run_identity(&self) -> &str {
        &self.run_identity
    }

    #[must_use]
    pub fn oracle_identity(&self) -> &str {
        &self.oracle_identity
    }

    #[must_use]
    pub fn evidence_tier(&self) -> &str {
        &self.evidence_tier
    }

    #[must_use]
    pub fn links(&self) -> &[SafeExternalLink] {
        &self.links
    }
}

fn oracle_summary(input: &ProvenanceInput<'_>) -> String {
    let Some(revision) = sanitized(input.oracle_revision) else {
        return "Oracle Unavailable".to_owned();
    };
    format!(
        "revision {revision}; compiler {}; preset {}",
        sanitized(input.oracle_compiler).unwrap_or(UNAVAILABLE),
        sanitized(input.oracle_preset).unwrap_or(UNAVAILABLE)
    )
}

fn fallback(maybe_value: Option<&str>) -> Box<str> {
    sanitized(maybe_value).unwrap_or(UNAVAILABLE).into()
}

fn sanitized(maybe_value: Option<&str>) -> Option<&str> {
    let value = maybe_value?.trim();
    (!value.is_empty()
        && value.len() <= MAXIMUM_VISIBLE_PROVENANCE_BYTES
        && value.is_ascii()
        && !value.chars().any(char::is_control))
    .then_some(value)
}

fn valid_commit(value: &str) -> bool {
    (7..=40).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
