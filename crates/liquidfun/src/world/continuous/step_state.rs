use super::{ContactManager, StepConfiguration};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::world) struct ContinuousStepKey {
    time_step_bits: u32,
    velocity_iterations: u32,
    position_iterations: u32,
}

impl ContinuousStepKey {
    pub(in crate::world) fn from_configuration(configuration: StepConfiguration) -> Self {
        Self {
            time_step_bits: configuration.time_step().to_bits(),
            velocity_iterations: configuration.velocity_iterations(),
            position_iterations: configuration.position_iterations(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::world) enum ContinuousStepKind {
    Fresh,
    Resumed,
}

#[derive(Debug, Clone, Copy, Default)]
pub(in crate::world) struct ContinuousStepState {
    maybe_pending: Option<ContinuousStepKey>,
}

impl ContinuousStepState {
    pub(in crate::world) const fn new() -> Self {
        Self {
            maybe_pending: None,
        }
    }

    pub(in crate::world) fn begin_step(
        &mut self,
        key: ContinuousStepKey,
        contact_manager: &mut ContactManager,
    ) -> ContinuousStepKind {
        if self.maybe_pending == Some(key) {
            return ContinuousStepKind::Resumed;
        }
        self.maybe_pending = None;
        contact_manager.reset_toi_state();
        ContinuousStepKind::Fresh
    }

    pub(in crate::world) fn mark_pending(&mut self, key: ContinuousStepKey) {
        self.maybe_pending = Some(key);
    }

    pub(in crate::world) fn invalidate(&mut self) {
        self.maybe_pending = None;
    }
}
