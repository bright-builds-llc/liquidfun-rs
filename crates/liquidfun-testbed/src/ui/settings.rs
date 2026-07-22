//! Validated staged run settings for the controller restart contract.

#![allow(
    missing_docs,
    reason = "closed field and error variants are named by the UI contract"
)]

use liquidfun_differential::RunSettingsInput;
use liquidfun_test_protocol::{FloatBits, ResolvedScenario, RunSettings};

use crate::controller_adapter::ControllerAction;

pub const MODULE_NAME: &str = "settings";
pub const TIMESTEP_GUIDANCE: &str = "Enter a finite timestep greater than 0";
pub const ITERATION_GUIDANCE: &str = "Enter an integer from 1 to 1024";
pub const APPLY_LABEL: &str = "Apply & Restart";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsField {
    Timestep,
    VelocityIterations,
    PositionIterations,
    ParticleIterations,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FieldDraft {
    text: String,
    maybe_error: Option<&'static str>,
}

impl FieldDraft {
    fn valid(text: String) -> Self {
        Self {
            text,
            maybe_error: None,
        }
    }
}

/// Staged settings retain the last accepted value when a draft fails on blur or Enter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsEditor {
    active: RunSettings,
    accepted: RunSettings,
    timestep: FieldDraft,
    velocity_iterations: FieldDraft,
    position_iterations: FieldDraft,
    particle_iterations: FieldDraft,
}

impl SettingsEditor {
    #[must_use]
    pub fn new(active: RunSettings) -> Self {
        Self {
            active,
            accepted: active,
            timestep: FieldDraft::valid(active.timestep_bits().to_f32().to_string()),
            velocity_iterations: FieldDraft::valid(active.velocity_iterations().to_string()),
            position_iterations: FieldDraft::valid(active.position_iterations().to_string()),
            particle_iterations: FieldDraft::valid(active.particle_iterations().to_string()),
        }
    }

    pub fn edit(&mut self, field: SettingsField, text: impl Into<String>) {
        let draft = self.draft_mut(field);
        draft.text = text.into();
        draft.maybe_error = None;
    }

    /// Parses one field on blur or Enter without changing the last accepted value on failure.
    pub fn commit(&mut self, field: SettingsField) {
        match field {
            SettingsField::Timestep => self.commit_timestep(),
            SettingsField::VelocityIterations
            | SettingsField::PositionIterations
            | SettingsField::ParticleIterations => self.commit_iterations(field),
        }
    }

    #[must_use]
    pub fn text(&self, field: SettingsField) -> &str {
        &self.draft(field).text
    }

    #[must_use]
    pub fn maybe_error(&self, field: SettingsField) -> Option<&'static str> {
        self.draft(field).maybe_error
    }

    #[must_use]
    pub const fn accepted(&self) -> RunSettings {
        self.accepted
    }

    #[must_use]
    pub fn apply_enabled(&self) -> bool {
        self.timestep.maybe_error.is_none()
            && self.velocity_iterations.maybe_error.is_none()
            && self.position_iterations.maybe_error.is_none()
            && self.particle_iterations.maybe_error.is_none()
            && self.accepted != self.active
    }

    /// Produces the only settings effect after every field is valid and changed.
    #[must_use]
    pub fn maybe_apply_action(&self, resolved: ResolvedScenario) -> Option<ControllerAction> {
        if !self.apply_enabled() {
            return None;
        }
        Some(ControllerAction::ApplySettingsAndRestart {
            settings: RunSettingsInput::new(
                self.accepted.timestep_bits(),
                self.accepted.velocity_iterations(),
                self.accepted.position_iterations(),
                self.accepted.particle_iterations(),
            ),
            resolved,
        })
    }

    fn commit_timestep(&mut self) {
        let maybe_value = self.timestep.text.parse::<f32>().ok();
        let Some(value) = maybe_value.filter(|value| value.is_finite() && *value > 0.0) else {
            self.timestep.maybe_error = Some(TIMESTEP_GUIDANCE);
            return;
        };
        let candidate = RunSettings::new(
            FloatBits::from_f32(value),
            self.accepted.velocity_iterations(),
            self.accepted.position_iterations(),
            self.accepted.particle_iterations(),
        );
        let Ok(accepted) = candidate else {
            self.timestep.maybe_error = Some(TIMESTEP_GUIDANCE);
            return;
        };
        self.accepted = accepted;
        self.timestep.maybe_error = None;
    }

    fn commit_iterations(&mut self, field: SettingsField) {
        let maybe_value = self.draft(field).text.parse::<u32>().ok();
        let Some(value) = maybe_value.filter(|value| (1..=1024).contains(value)) else {
            self.draft_mut(field).maybe_error = Some(ITERATION_GUIDANCE);
            return;
        };
        let mut velocity = self.accepted.velocity_iterations();
        let mut position = self.accepted.position_iterations();
        let mut particle = self.accepted.particle_iterations();
        match field {
            SettingsField::VelocityIterations => velocity = value,
            SettingsField::PositionIterations => position = value,
            SettingsField::ParticleIterations => particle = value,
            SettingsField::Timestep => return,
        }
        let candidate =
            RunSettings::new(self.accepted.timestep_bits(), velocity, position, particle);
        let Ok(accepted) = candidate else {
            self.draft_mut(field).maybe_error = Some(ITERATION_GUIDANCE);
            return;
        };
        self.accepted = accepted;
        self.draft_mut(field).maybe_error = None;
    }

    const fn draft(&self, field: SettingsField) -> &FieldDraft {
        match field {
            SettingsField::Timestep => &self.timestep,
            SettingsField::VelocityIterations => &self.velocity_iterations,
            SettingsField::PositionIterations => &self.position_iterations,
            SettingsField::ParticleIterations => &self.particle_iterations,
        }
    }

    const fn draft_mut(&mut self, field: SettingsField) -> &mut FieldDraft {
        match field {
            SettingsField::Timestep => &mut self.timestep,
            SettingsField::VelocityIterations => &mut self.velocity_iterations,
            SettingsField::PositionIterations => &mut self.position_iterations,
            SettingsField::ParticleIterations => &mut self.particle_iterations,
        }
    }
}
