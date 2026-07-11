//! Private native-Rust execution seams for empty-world traces and pure math probes.

use liquidfun_test_protocol::{
    BuildIdentity, BuildIdentityError, BuildIdentityFields, CheckpointRecord, EngineKind,
    FloatBits, HarnessLimits, Phase4BuildIdentityFields, ScenarioRequestRecord, Sha256Hex,
    TraceBegin, TraceEnd, TraceHashError, TraceRecord, TraceValidationError, TraceValidator,
    ValidatedTrace, WorldCounts, trace_payload_sha256,
};
use sha2::{Digest, Sha256};

use crate::{MathProbeExecutionError, NativeMathProbeExecutor};

/// Failure while constructing or executing the private native adapter.
#[derive(Debug, thiserror::Error)]
pub enum EmptyWorldAdapterError {
    /// Native build identity fields failed protocol validation.
    #[error(transparent)]
    BuildIdentity(#[from] BuildIdentityError),
    /// Typed trace construction or final validation failed.
    #[error(transparent)]
    TraceValidation(#[from] TraceValidationError),
    /// Ordered checkpoint payload hashing failed.
    #[error(transparent)]
    TraceHash(#[from] TraceHashError),
    /// A bounded count cannot be represented by its wire type.
    #[error("native trace count cannot be represented by its wire type")]
    CountOverflow,
    /// The reset epoch exhausted its representation.
    #[error("native adapter reset epoch overflowed")]
    ResetEpochOverflow,
}

/// Private executor for the deliberately narrow Phase-2 empty-world contract.
pub struct EmptyWorldAdapter {
    build_identity: BuildIdentity,
    reset_epoch: u64,
}

impl EmptyWorldAdapter {
    /// Executes a validated pure math request without creating or mutating world state.
    pub fn execute_math_probe(
        request: &liquidfun_test_protocol::MathProbeRequestRecord,
    ) -> Result<Box<[liquidfun_test_protocol::MathProbeResult]>, MathProbeExecutionError> {
        NativeMathProbeExecutor::execute(request)
    }

    /// Creates a native adapter bound to the selected comparison-oracle revision.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyWorldAdapterError`] when the supplied revision or generated native build
    /// identity is invalid.
    pub fn new(
        comparison_oracle_revision: impl Into<String>,
    ) -> Result<Self, EmptyWorldAdapterError> {
        let adapter_revision = env!("CARGO_PKG_VERSION");
        let adapter_content_sha256 =
            native_adapter_content_sha256(include_bytes!("rust_adapter.rs"));
        let target = format!(
            "target={};host={}",
            env!("LIQUIDFUN_NATIVE_TARGET"),
            env!("LIQUIDFUN_NATIVE_HOST")
        );
        let compile_flags = format!(
            "features={};encoded_rustflags={}",
            env!("LIQUIDFUN_NATIVE_FEATURES"),
            env!("LIQUIDFUN_NATIVE_ENCODED_RUSTFLAGS")
        );
        let link_flags = format!(
            "encoded_rustflags={}",
            env!("LIQUIDFUN_NATIVE_ENCODED_RUSTFLAGS")
        );
        let feature_set = format!(
            "features={};target_cpu={};target_features={};encoded_rustflags={}",
            env!("LIQUIDFUN_NATIVE_FEATURES"),
            env!("LIQUIDFUN_NATIVE_TARGET_CPU"),
            env!("LIQUIDFUN_NATIVE_TARGET_FEATURES"),
            env!("LIQUIDFUN_NATIVE_ENCODED_RUSTFLAGS")
        );
        let compile_descriptor = format!(
            "rustc={};target={};host={};profile={};optimization={};{}",
            env!("LIQUIDFUN_NATIVE_RUSTC_VV"),
            env!("LIQUIDFUN_NATIVE_TARGET"),
            env!("LIQUIDFUN_NATIVE_HOST"),
            env!("LIQUIDFUN_NATIVE_PROFILE"),
            env!("LIQUIDFUN_NATIVE_OPTIMIZATION"),
            feature_set
        );
        let compile_command_sha256 =
            Sha256Hex::from_digest(Sha256::digest(compile_descriptor).into());
        let phase4 = Phase4BuildIdentityFields::new(
            compile_command_sha256.as_str(),
            "rustc",
            env!("LIQUIDFUN_NATIVE_RUSTC_VERSION"),
            env!("LIQUIDFUN_NATIVE_TARGET"),
            env!("LIQUIDFUN_NATIVE_TARGET_CPU"),
            env!("LIQUIDFUN_NATIVE_TARGET_FEATURES"),
            "<none>",
            env!("LIQUIDFUN_NATIVE_OPTIMIZATION"),
            "precise",
            "off",
            "ieee",
            feature_set,
            env!("LIQUIDFUN_NATIVE_TARGET_OS"),
            env!("LIQUIDFUN_NATIVE_LIBC"),
            env!("LIQUIDFUN_NATIVE_LIBM"),
            native_rounding_mode(),
            native_gradual_underflow(),
        );
        let fields = BuildIdentityFields::new(
            comparison_oracle_revision,
            adapter_revision,
            adapter_content_sha256.as_str(),
            "native-rust",
            "rustc",
            env!("LIQUIDFUN_NATIVE_RUSTC_VERSION"),
            target,
            env!("LIQUIDFUN_NATIVE_PROFILE"),
            compile_flags,
            link_flags,
            "none",
        )
        .with_phase4(phase4);

        Ok(Self {
            build_identity: BuildIdentity::new(fields)?,
            reset_epoch: 0,
        })
    }

    /// Returns the native build identity carried by every emitted trace record.
    #[must_use]
    pub const fn build_identity(&self) -> &BuildIdentity {
        &self.build_identity
    }

    /// Executes one already-validated empty-world request and proves local state reset.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyWorldAdapterError`] if typed record construction, hashing, reset-epoch
    /// advancement, or complete trace validation fails.
    pub fn execute(
        &mut self,
        request: &ScenarioRequestRecord,
    ) -> Result<ValidatedTrace, EmptyWorldAdapterError> {
        let begin = TraceBegin::for_request(request, EngineKind::NativeRust, &self.build_identity)?;
        let checkpoints = execute_checkpoints(request, &self.build_identity)?;
        let next_reset_epoch = self
            .reset_epoch
            .checked_add(1)
            .ok_or(EmptyWorldAdapterError::ResetEpochOverflow)?;
        let checkpoint_count =
            u32::try_from(checkpoints.len()).map_err(|_| EmptyWorldAdapterError::CountOverflow)?;
        let payload_hash = trace_payload_sha256(&checkpoints)?;
        let end = TraceEnd::new(
            request.request_id().clone(),
            checkpoint_count,
            payload_hash,
            next_reset_epoch,
            true,
            self.build_identity.identity_sha256().clone(),
        );
        let records = std::iter::once(TraceRecord::Begin(begin))
            .chain(checkpoints.into_iter().map(TraceRecord::Checkpoint))
            .chain(std::iter::once(TraceRecord::End(end)))
            .collect();
        let trace = TraceValidator::validate(
            request,
            &self.build_identity,
            next_reset_epoch,
            records,
            &HarnessLimits::phase2_default_v1(),
        )?;
        self.reset_epoch = next_reset_epoch;
        Ok(trace)
    }
}

fn native_rounding_mode() -> &'static str {
    let half_ulp = f32::from_bits(0x3380_0000);
    let ties_even = (1.0_f32 + half_ulp).to_bits() == 1.0_f32.to_bits();
    let odd_rounds_up = (f32::from_bits(0x3f80_0001) + half_ulp).to_bits() == 0x3f80_0002;
    if ties_even && odd_rounds_up {
        "nearest_ties_even"
    } else {
        "unsupported"
    }
}

fn native_gradual_underflow() -> bool {
    (f32::MIN_POSITIVE * 0.5_f32).to_bits() == 0x0040_0000
}

struct EmptyWorldState {
    _gravity: [FloatBits; 2],
    simulation_time: f32,
}

impl EmptyWorldState {
    const fn new(gravity: [FloatBits; 2]) -> Self {
        Self {
            _gravity: gravity,
            simulation_time: 0.0,
        }
    }

    fn step(&mut self, timestep: FloatBits) -> FloatBits {
        self.simulation_time += timestep.to_f32();
        FloatBits::from_f32(self.simulation_time)
    }
}

fn execute_checkpoints(
    request: &ScenarioRequestRecord,
    identity: &BuildIdentity,
) -> Result<Vec<CheckpointRecord>, EmptyWorldAdapterError> {
    let scenario = request.scenario();
    let mut state = EmptyWorldState::new([scenario.gravity_x_bits(), scenario.gravity_y_bits()]);
    let mut checkpoints = Vec::with_capacity(scenario.checkpoints().len());

    for command in scenario.commands() {
        let simulation_time_bits = state.step(command.timestep_bits());
        for requested in scenario
            .checkpoints()
            .iter()
            .filter(|checkpoint| checkpoint.after_command_id() == command.command_id())
        {
            let ordinal = u32::try_from(checkpoints.len())
                .map_err(|_| EmptyWorldAdapterError::CountOverflow)?;
            checkpoints.push(CheckpointRecord::new(
                request.request_id().clone(),
                requested.checkpoint_id().clone(),
                ordinal,
                requested.phase(),
                simulation_time_bits,
                WorldCounts::zero(),
                identity.identity_sha256().clone(),
            )?);
        }
    }

    Ok(checkpoints)
}

fn native_adapter_content_sha256(source: &[u8]) -> Sha256Hex {
    Sha256Hex::from_digest(Sha256::digest(source).into())
}

#[cfg(test)]
mod tests {
    use super::native_adapter_content_sha256;

    #[test]
    fn adapter_content_digest_changes_with_source_input() {
        // Arrange
        let original = b"native adapter source v1";
        let changed = b"native adapter source v2";

        // Act
        let original_digest = native_adapter_content_sha256(original);
        let changed_digest = native_adapter_content_sha256(changed);

        // Assert
        assert_ne!(original_digest, changed_digest);
    }
}
