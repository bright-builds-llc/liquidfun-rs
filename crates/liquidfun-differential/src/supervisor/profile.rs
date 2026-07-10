//! Named immutable oracle session lifecycle and resource profiles.

use liquidfun_test_protocol::HarnessLimits;

/// Named immutable lifecycle and resource configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionProfile {
    /// One process and one request for maximum isolation.
    OneShot,
    /// Sequential finite process reuse with periodic cycling.
    Reuse,
    /// One fail-fast sanitizer request.
    Sanitizer,
}

impl SessionProfile {
    pub(super) fn limits(self) -> HarnessLimits {
        match self {
            Self::OneShot => HarnessLimits::phase2_default_v1(),
            Self::Reuse => HarnessLimits::phase2_reuse_v1(),
            Self::Sanitizer => HarnessLimits::phase2_sanitizer_v1(),
        }
    }

    pub(super) const fn keeps_process(self) -> bool {
        matches!(self, Self::Reuse)
    }
}
