//! Exact validated oracle bytes retained for reviewed fixture staging.

use liquidfun_test_protocol::ValidatedTrace;

/// One fully validated oracle trace together with its exact handshake and JSONL record bytes.
pub struct CapturedOracleTrace {
    pub(super) trace: ValidatedTrace,
    pub(super) jsonl: Box<[u8]>,
}

impl CapturedOracleTrace {
    /// Returns the validated semantic trace.
    #[must_use]
    pub const fn trace(&self) -> &ValidatedTrace {
        &self.trace
    }

    /// Returns the exact newline-complete handshake and trace bytes emitted by the oracle.
    #[must_use]
    pub fn jsonl(&self) -> &[u8] {
        &self.jsonl
    }

    pub(super) fn into_trace(self) -> ValidatedTrace {
        self.trace
    }
}
