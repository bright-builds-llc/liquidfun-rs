use liquidfun_test_protocol::{CheckpointDeclaration, CheckpointId, ScenarioActionId};

use super::SessionState;

/// Monotonic identity of one frontend-submitted command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionCommandId(pub(super) u64);

impl SessionCommandId {
    /// Creates a nonzero command identity.
    ///
    /// # Errors
    ///
    /// Returns [`SessionCommandIdError`] for zero, which is reserved as an invalid sentinel.
    pub const fn new(value: u64) -> Result<Self, SessionCommandIdError> {
        if value == 0 {
            return Err(SessionCommandIdError);
        }
        Ok(Self(value))
    }

    /// Returns the validated ordinal.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Rejection of the reserved zero command identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("session command identity must be nonzero")]
pub struct SessionCommandIdError;

/// Stable checkpoint identity bound to both action and logical-step ordinals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCheckpointIdentity {
    checkpoint_id: CheckpointId,
    after_action_id: ScenarioActionId,
    logical_step: u32,
}

impl SessionCheckpointIdentity {
    pub(super) fn from_declaration(declaration: &CheckpointDeclaration) -> Self {
        Self {
            checkpoint_id: declaration.checkpoint_id().clone(),
            after_action_id: declaration.after_action_id().clone(),
            logical_step: declaration.logical_step(),
        }
    }

    /// Returns the stable checkpoint ID.
    #[must_use]
    pub const fn checkpoint_id(&self) -> &CheckpointId {
        &self.checkpoint_id
    }

    /// Returns the stable action boundary captured by this checkpoint.
    #[must_use]
    pub const fn after_action_id(&self) -> &ScenarioActionId {
        &self.after_action_id
    }

    /// Returns the explicit one-based logical-step ordinal.
    #[must_use]
    pub const fn logical_step(&self) -> u32 {
        self.logical_step
    }
}

/// One successful owned semantic capture and its controller-bound identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCapture<T> {
    pub(super) identity: SessionCheckpointIdentity,
    pub(super) value: T,
}

impl<T> SessionCapture<T> {
    /// Returns the stable deterministic capture identity.
    #[must_use]
    pub const fn identity(&self) -> &SessionCheckpointIdentity {
        &self.identity
    }

    /// Returns the backend-owned semantic checkpoint value.
    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }
}

/// State returned after one accepted command settles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionCommandOutcome {
    pub(super) state: SessionState,
}

impl SessionCommandOutcome {
    /// Returns the settled controller state.
    #[must_use]
    pub const fn state(self) -> SessionState {
        self.state
    }
}
