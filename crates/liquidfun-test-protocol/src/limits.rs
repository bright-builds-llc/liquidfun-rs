use std::time::Duration;

use sha2::{Digest, Sha256};

use crate::Sha256Hex;

/// Immutable, reviewed resource and lifecycle limits for one harness profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessLimits {
    profile_id: &'static str,
    input_record_bytes: usize,
    json_nesting_depth: usize,
    decoded_string_bytes: usize,
    typed_id_bytes: usize,
    entity_definitions: usize,
    commands: usize,
    checkpoints: usize,
    observables_per_checkpoint: usize,
    output_record_bytes: usize,
    complete_trace_bytes: usize,
    retained_stderr_bytes: usize,
    total_child_output_bytes: usize,
    startup_timeout: Duration,
    request_timeout: Duration,
    request_budget: usize,
}

impl HarnessLimits {
    /// Returns the one-shot phase-2 profile.
    #[must_use]
    pub const fn phase2_default_v1() -> Self {
        Self::phase2("phase2-default-v1", 1)
    }

    /// Returns the bounded reusable-corpus phase-2 profile.
    #[must_use]
    pub const fn phase2_reuse_v1() -> Self {
        Self::phase2("phase2-reuse-v1", 100)
    }

    /// Returns the isolated sanitizer phase-2 profile.
    #[must_use]
    pub const fn phase2_sanitizer_v1() -> Self {
        Self::phase2("phase2-sanitizer-v1", 1)
    }

    const fn phase2(profile_id: &'static str, request_budget: usize) -> Self {
        Self {
            profile_id,
            input_record_bytes: 1024 * 1024,
            json_nesting_depth: 32,
            decoded_string_bytes: 4 * 1024,
            typed_id_bytes: 128,
            entity_definitions: 4_096,
            commands: 4_096,
            checkpoints: 4_096,
            observables_per_checkpoint: 128,
            output_record_bytes: 1024 * 1024,
            complete_trace_bytes: 32 * 1024 * 1024,
            retained_stderr_bytes: 256 * 1024,
            total_child_output_bytes: 64 * 1024 * 1024,
            startup_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(10),
            request_budget,
        }
    }

    /// Returns the stable profile identifier.
    #[must_use]
    pub const fn profile_id(&self) -> &'static str {
        self.profile_id
    }

    /// Returns the maximum input JSONL record size.
    #[must_use]
    pub const fn input_record_bytes(&self) -> usize {
        self.input_record_bytes
    }

    /// Returns the maximum JSON nesting depth.
    #[must_use]
    pub const fn json_nesting_depth(&self) -> usize {
        self.json_nesting_depth
    }

    /// Returns the maximum general decoded string size.
    #[must_use]
    pub const fn decoded_string_bytes(&self) -> usize {
        self.decoded_string_bytes
    }

    /// Returns the maximum typed identifier size.
    #[must_use]
    pub const fn typed_id_bytes(&self) -> usize {
        self.typed_id_bytes
    }

    /// Returns the maximum entity-definition count.
    #[must_use]
    pub const fn entity_definitions(&self) -> usize {
        self.entity_definitions
    }

    /// Returns the maximum command count.
    #[must_use]
    pub const fn commands(&self) -> usize {
        self.commands
    }

    /// Returns the maximum checkpoint count.
    #[must_use]
    pub const fn checkpoints(&self) -> usize {
        self.checkpoints
    }

    /// Returns the maximum observables per checkpoint.
    #[must_use]
    pub const fn observables_per_checkpoint(&self) -> usize {
        self.observables_per_checkpoint
    }

    /// Returns the maximum output JSONL record size.
    #[must_use]
    pub const fn output_record_bytes(&self) -> usize {
        self.output_record_bytes
    }

    /// Returns the maximum complete trace size.
    #[must_use]
    pub const fn complete_trace_bytes(&self) -> usize {
        self.complete_trace_bytes
    }

    /// Returns the maximum retained stderr size.
    #[must_use]
    pub const fn retained_stderr_bytes(&self) -> usize {
        self.retained_stderr_bytes
    }

    /// Returns the maximum total child output per request.
    #[must_use]
    pub const fn total_child_output_bytes(&self) -> usize {
        self.total_child_output_bytes
    }

    /// Returns the startup deadline.
    #[must_use]
    pub const fn startup_timeout(&self) -> Duration {
        self.startup_timeout
    }

    /// Returns the phase-2 request deadline.
    #[must_use]
    pub const fn request_timeout(&self) -> Duration {
        self.request_timeout
    }

    /// Returns the maximum requests handled by one process.
    #[must_use]
    pub const fn request_budget(&self) -> usize {
        self.request_budget
    }

    /// Hashes every profile field into a stable evidence identity.
    #[must_use]
    pub fn profile_sha256(&self) -> Sha256Hex {
        let canonical = format!(
            "profile_id={}\ninput_record_bytes={}\njson_nesting_depth={}\n\
             decoded_string_bytes={}\ntyped_id_bytes={}\nentity_definitions={}\ncommands={}\n\
             checkpoints={}\nobservables_per_checkpoint={}\noutput_record_bytes={}\n\
             complete_trace_bytes={}\nretained_stderr_bytes={}\ntotal_child_output_bytes={}\n\
             startup_timeout_ms={}\nrequest_timeout_ms={}\nrequest_budget={}\n",
            self.profile_id,
            self.input_record_bytes,
            self.json_nesting_depth,
            self.decoded_string_bytes,
            self.typed_id_bytes,
            self.entity_definitions,
            self.commands,
            self.checkpoints,
            self.observables_per_checkpoint,
            self.output_record_bytes,
            self.complete_trace_bytes,
            self.retained_stderr_bytes,
            self.total_child_output_bytes,
            self.startup_timeout.as_millis(),
            self.request_timeout.as_millis(),
            self.request_budget,
        );
        Sha256Hex::from_digest(Sha256::digest(canonical.as_bytes()).into())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::HarnessLimits;

    #[test]
    fn phase2_default_profile_exposes_reviewed_boundaries() {
        // Arrange and Act
        let limits = HarnessLimits::phase2_default_v1();

        // Assert
        assert_eq!(limits.profile_id(), "phase2-default-v1");
        assert_eq!(limits.input_record_bytes(), 1024 * 1024);
        assert_eq!(limits.json_nesting_depth(), 32);
        assert_eq!(limits.decoded_string_bytes(), 4 * 1024);
        assert_eq!(limits.typed_id_bytes(), 128);
        assert_eq!(limits.entity_definitions(), 4_096);
        assert_eq!(limits.commands(), 4_096);
        assert_eq!(limits.checkpoints(), 4_096);
        assert_eq!(limits.observables_per_checkpoint(), 128);
        assert_eq!(limits.output_record_bytes(), 1024 * 1024);
        assert_eq!(limits.complete_trace_bytes(), 32 * 1024 * 1024);
        assert_eq!(limits.retained_stderr_bytes(), 256 * 1024);
        assert_eq!(limits.total_child_output_bytes(), 64 * 1024 * 1024);
        assert_eq!(limits.startup_timeout(), Duration::from_secs(5));
        assert_eq!(limits.request_timeout(), Duration::from_secs(10));
        assert_eq!(limits.request_budget(), 1);
    }

    #[test]
    fn reviewed_profiles_have_stable_distinct_hashes_and_budgets() {
        // Arrange
        let default = HarnessLimits::phase2_default_v1();
        let reuse = HarnessLimits::phase2_reuse_v1();
        let sanitizer = HarnessLimits::phase2_sanitizer_v1();

        // Act
        let default_hash = default.profile_sha256();

        // Assert
        assert_eq!(
            default_hash,
            HarnessLimits::phase2_default_v1().profile_sha256()
        );
        assert_ne!(default_hash, reuse.profile_sha256());
        assert_ne!(default_hash, sanitizer.profile_sha256());
        assert_eq!(reuse.request_budget(), 100);
        assert_eq!(sanitizer.request_budget(), 1);
    }
}
