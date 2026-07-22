//! Focused deterministic fixtures for the controller UI contract.

use liquidfun::{
    DebugColor, DebugLayer, DebugOwnerKey, DebugPrimitive, DebugPrimitiveKey, DebugPrimitiveKind,
    DebugPrimitiveMetadata, DebugStroke, math::Vec2,
};
use liquidfun_differential::{
    SessionBackend, SessionBackendError, SessionCheckpointIdentity, SessionCommand,
    SessionController,
};
use liquidfun_test_protocol::{
    CatalogSlug, FloatBits, ResolveRequest, RunSettings, ScheduledAction, resolve_catalog,
    scenarios::rigid,
};

#[derive(Debug, Default)]
pub(crate) struct RecordingBackend {
    pub(crate) ticks: usize,
    pub(crate) captures: usize,
}

impl SessionBackend for RecordingBackend {
    type Checkpoint = u32;

    fn create_session(
        &mut self,
        _resolved: &liquidfun_test_protocol::ResolvedScenario,
    ) -> Result<(), SessionBackendError> {
        Ok(())
    }

    fn destroy_session(&mut self) {}

    fn execute_action(&mut self, _action: &ScheduledAction) -> Result<(), SessionBackendError> {
        self.ticks += 1;
        Ok(())
    }

    fn capture_checkpoint(
        &mut self,
        checkpoint: &SessionCheckpointIdentity,
    ) -> Result<Self::Checkpoint, SessionBackendError> {
        self.captures += 1;
        Ok(checkpoint.logical_step())
    }
}

pub(crate) fn settings() -> RunSettings {
    RunSettings::new(FloatBits::from_f32(1.0 / 60.0), 8, 3, 2)
        .expect("fixture settings are finite and in range")
}

pub(crate) fn resolved(settings: RunSettings) -> liquidfun_test_protocol::ResolvedScenario {
    let definitions = rigid::definitions().expect("reviewed rigid definitions are valid");
    let slug = CatalogSlug::new("rigid-contact-lifecycle").expect("fixture catalog slug is valid");
    let request = ResolveRequest::new(slug, None, settings);
    resolve_catalog(&definitions, &request).expect("fixture scenario resolves")
}

pub(crate) fn selected_controller() -> SessionController<RecordingBackend> {
    let mut controller = SessionController::new(RecordingBackend::default());
    let command_id = controller
        .next_command_id()
        .expect("new controller accepts command one");
    controller
        .submit(
            command_id,
            SessionCommand::Select {
                resolved: resolved(settings()),
            },
        )
        .expect("fixture selection succeeds");
    controller
}

pub(crate) fn submit(
    controller: &mut SessionController<RecordingBackend>,
    command: SessionCommand,
) {
    let command_id = controller
        .next_command_id()
        .expect("fixture command counter remains available");
    controller
        .submit(command_id, command)
        .expect("fixture controller command succeeds");
}

pub(crate) fn point(layer: DebugLayer, ordinal: u32) -> DebugPrimitive {
    let key = DebugPrimitiveKey::new(
        DebugOwnerKey::World,
        layer,
        DebugPrimitiveKind::Point,
        0,
        ordinal,
    );
    let stroke = DebugStroke::new(DebugColor::rgba(139, 148, 158, 255), 0.02)
        .expect("fixture stroke is finite");
    DebugPrimitive::Point {
        metadata: DebugPrimitiveMetadata::new(key, stroke, None),
        position: Vec2::ZERO,
        radius: 0.1,
    }
}

pub(crate) fn arrow(layer: DebugLayer, ordinal: u32) -> DebugPrimitive {
    let key = DebugPrimitiveKey::new(
        DebugOwnerKey::World,
        layer,
        DebugPrimitiveKind::Arrow,
        0,
        ordinal,
    );
    let stroke = DebugStroke::new(DebugColor::rgba(210, 153, 34, 255), 0.02)
        .expect("fixture stroke is finite");
    DebugPrimitive::Arrow {
        metadata: DebugPrimitiveMetadata::new(key, stroke, None),
        start: Vec2::ZERO,
        end: Vec2::new(0.0, 1.0),
    }
}
