//! Native rigid execution boundary and typed failures.

use liquidfun_test_protocol::{
    RigidWorldDecodeError, RigidWorldRequestRecord, RigidWorldResultRecord, RigidWorldWitnessFamily,
};

use super::{execute_timeline, validate_native_rigid_world_result};

/// Typed failure while mapping a validated rigid timeline onto native world APIs.
#[derive(Debug, thiserror::Error)]
pub enum NativeRigidWorldError {
    /// A semantic action could not resolve or execute through the checked world API.
    #[error("native rigid action `{action_id}` failed: {message}")]
    Action {
        /// Stable action identity.
        action_id: Box<str>,
        /// Bounded checked-world diagnostic.
        message: Box<str>,
    },
    /// Produced state disagreed with the validated declaration contract.
    #[error("native rigid declaration disagreement at `{checkpoint_id}`: {message}")]
    Declaration {
        /// Stable checkpoint identity.
        checkpoint_id: Box<str>,
        /// Bounded mismatch diagnostic.
        message: Box<str>,
    },
    /// Result construction or protocol validation rejected the aggregate.
    #[error(transparent)]
    Result(#[from] RigidWorldDecodeError),
    /// A completed timeline retained native world state.
    #[error("native rigid timeline `{family:?}` failed terminal reset proof")]
    Reset {
        /// Timeline family that retained state.
        family: RigidWorldWitnessFamily,
    },
    /// Native execution panicked before a complete result could be accepted.
    #[error("native rigid timeline `{timeline_id}` panicked; no partial result was emitted")]
    Panic {
        /// Stable timeline identity used to identify the failed request member.
        timeline_id: Box<str>,
    },
}

/// Stateless native executor for one validated Phase 6 request.
pub struct NativeRigidWorldExecutor;

impl NativeRigidWorldExecutor {
    /// Executes every timeline through a fresh native world.
    ///
    /// # Errors
    ///
    /// Returns [`NativeRigidWorldError`] without an accepted partial result when semantic
    /// resolution, checked world execution, declaration validation, or reset proof fails.
    pub fn execute(
        request: &RigidWorldRequestRecord,
    ) -> Result<RigidWorldResultRecord, NativeRigidWorldError> {
        let timelines = request
            .scenario()
            .timelines()
            .iter()
            .map(execute_timeline)
            .collect::<Result<Vec<_>, _>>()?;
        let result = RigidWorldResultRecord::new(
            request.request_id().clone(),
            request.scenario().scenario_id().clone(),
            timelines,
        )?;
        validate_native_rigid_world_result(request, &result)?;
        Ok(result)
    }
}
