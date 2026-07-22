//! Concrete backend bindings kept behind the renderer-neutral session trait.

use crate::NativeCatalogBackend;

use super::{SessionBackend, SessionBackendError, SessionCheckpointIdentity};
use liquidfun_test_protocol::{CanonicalCheckpoint, ResolvedScenario, ScheduledAction};

impl SessionBackend for NativeCatalogBackend {
    type Checkpoint = CanonicalCheckpoint;

    fn create_session(&mut self, resolved: &ResolvedScenario) -> Result<(), SessionBackendError> {
        self.create(resolved)
    }

    fn destroy_session(&mut self) {
        self.destroy();
    }

    fn execute_action(&mut self, action: &ScheduledAction) -> Result<(), SessionBackendError> {
        self.execute(action)
    }

    fn capture_checkpoint(
        &mut self,
        checkpoint: &SessionCheckpointIdentity,
    ) -> Result<Self::Checkpoint, SessionBackendError> {
        self.capture(checkpoint)
    }
}
