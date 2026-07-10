//! Private native-Rust execution seam for the Phase-2 empty-world scenario.

use liquidfun_test_protocol::{
    BuildIdentity, BuildIdentityError, BuildIdentityFields, CheckpointRecord, EngineKind,
    FloatBits, HarnessLimits, ScenarioRequestRecord, Sha256Hex, TraceBegin, TraceEnd,
    TraceHashError, TraceRecord, TraceValidationError, TraceValidator, ValidatedTrace, WorldCounts,
    trace_payload_sha256,
};
use sha2::{Digest, Sha256};

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
        let adapter_content_sha256 = native_adapter_content_sha256(adapter_revision);
        let target = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
        let build_type = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };
        let fields = BuildIdentityFields::new(
            comparison_oracle_revision,
            adapter_revision,
            adapter_content_sha256.as_str(),
            "native-rust",
            "rustc",
            "repository-toolchain",
            target,
            build_type,
            "rust-default",
            "cargo-default",
            "none",
        );

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

fn native_adapter_content_sha256(adapter_revision: &str) -> Sha256Hex {
    let mut hasher = Sha256::new();
    hasher.update(env!("CARGO_PKG_NAME").as_bytes());
    hasher.update(adapter_revision.as_bytes());
    hasher.update(b"phase2-empty-world-adapter-v1");
    Sha256Hex::from_digest(hasher.finalize().into())
}
