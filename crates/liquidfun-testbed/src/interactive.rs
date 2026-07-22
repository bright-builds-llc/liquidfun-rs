//! Live catalog-backed session model for the private interactive testbed.

use std::time::Duration;

use liquidfun_differential::{
    NativeCatalogBackend, RunSettingsInput, SessionController, SessionControllerError, SessionState,
};
use liquidfun_test_protocol::{
    CanonicalCheckpoint, CatalogErrorKind, CatalogSlug, CheckpointId, ResolveRequest,
    ResolvedScenario, RunSettings, ScenarioCatalog, resolve_catalog, reviewed_scenario_catalog,
};

use crate::app::CatalogSelection;
use crate::controller_adapter::{ControllerAction, ControllerAdapter, ControllerAdapterError};
use crate::ui::{ScenarioBrowser, ScenarioBrowserError, ScenarioRow};

/// Deterministic seed supplied when a reviewed definition requires one and the UI has not.
pub const DEFAULT_REQUIRED_SEED: u64 = 0;

/// Maximum logical actions performed by one render-loop update.
pub const MAXIMUM_STEPS_PER_UPDATE: u32 = 8;

/// Bounded failures safe for presentation by the private launcher.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InteractiveTestbedError {
    /// The reviewed catalog failed validation or resolution.
    #[error("scenario catalog failure: {0:?}")]
    Catalog(CatalogErrorKind),
    /// Browser projection or bounded search failed.
    #[error(transparent)]
    Browser(ScenarioBrowserError),
    /// A visible row index was outside the current filtered projection.
    #[error("visible scenario row is unavailable")]
    InvalidVisibleRow,
    /// A resolved plan disagreed with the selected stable scenario version.
    #[error("resolved scenario identity disagrees with the selected catalog row")]
    SelectionIdentityMismatch,
    /// The typed action was rejected before controller submission.
    #[error(transparent)]
    Adapter(ControllerAdapterError),
    /// The controller exhausted its monotonic command identities.
    #[error("session command identity space is exhausted")]
    CommandCounterExhausted,
    /// The renderer-neutral controller rejected the typed action.
    #[error(transparent)]
    Controller(SessionControllerError),
    /// Settings cannot be changed before a catalog scenario is selected.
    #[error("no scenario is selected")]
    NoSelection,
    /// No uncaptured checkpoint is declared at the current logical boundary.
    #[error("no canonical checkpoint is reachable at the current logical step")]
    NoReachableCheckpoint,
    /// The selected positive timestep cannot be represented by the monotonic clock duration.
    #[error("selected timestep is below the monotonic clock resolution")]
    TimestepBelowClockResolution,
}

impl From<ScenarioBrowserError> for InteractiveTestbedError {
    fn from(error: ScenarioBrowserError) -> Self {
        Self::Browser(error)
    }
}

impl From<ControllerAdapterError> for InteractiveTestbedError {
    fn from(error: ControllerAdapterError) -> Self {
        Self::Adapter(error)
    }
}

impl From<SessionControllerError> for InteractiveTestbedError {
    fn from(error: SessionControllerError) -> Self {
        Self::Controller(error)
    }
}

/// Catalog browser, typed command adapter, and native session owned by the live testbed.
pub struct InteractiveTestbed {
    catalog: ScenarioCatalog,
    browser: ScenarioBrowser,
    controller: SessionController<NativeCatalogBackend>,
    adapter: ControllerAdapter,
    maybe_current_selection: Option<CatalogSelection>,
    accumulated_time: Duration,
}

impl InteractiveTestbed {
    /// Loads the reviewed catalog and constructs an inactive native session owner.
    ///
    /// # Errors
    ///
    /// Returns a bounded catalog or browser projection failure.
    pub fn new() -> Result<Self, InteractiveTestbedError> {
        let catalog = reviewed_scenario_catalog()
            .map_err(|error| InteractiveTestbedError::Catalog(error.kind()))?;
        let browser = ScenarioBrowser::from_catalog(&catalog)?;
        Ok(Self {
            catalog,
            browser,
            controller: SessionController::new(NativeCatalogBackend::new()),
            adapter: ControllerAdapter::default(),
            maybe_current_selection: None,
            accumulated_time: Duration::ZERO,
        })
    }

    /// Returns filtered catalog rows in reviewed stable order.
    #[must_use]
    pub fn visible_rows(&self) -> &[ScenarioRow] {
        self.browser.visible_rows()
    }

    /// Applies bounded scenario search to the shared browser model.
    ///
    /// # Errors
    ///
    /// Returns [`InteractiveTestbedError::Browser`] for invalid search text.
    pub fn set_query(&mut self, query: &str) -> Result<(), InteractiveTestbedError> {
        self.browser.set_query(query).map_err(Into::into)
    }

    /// Resolves and selects one currently visible reviewed catalog row.
    ///
    /// # Errors
    ///
    /// Returns a bounded row, catalog, adapter, or controller failure.
    pub fn select_visible(&mut self, index: usize) -> Result<(), InteractiveTestbedError> {
        let selection = self
            .browser
            .visible_rows()
            .get(index)
            .ok_or(InteractiveTestbedError::InvalidVisibleRow)?
            .selection()
            .clone();
        let resolved = self.resolve_default_selection(&selection)?;
        self.perform(ControllerAction::Select(resolved))
    }

    /// Returns the exact selected slug, version, and resolved seed.
    #[must_use]
    pub const fn current_selection(&self) -> Option<&CatalogSelection> {
        self.maybe_current_selection.as_ref()
    }

    /// Returns the controller-owned immutable resolved plan.
    #[must_use]
    pub const fn selected(&self) -> Option<&ResolvedScenario> {
        self.controller.selected()
    }

    /// Returns the exact active timestep and solver iteration settings.
    #[must_use]
    pub fn selected_settings(&self) -> Option<RunSettings> {
        self.controller
            .selected()
            .map(|resolved| resolved.identity().settings())
    }

    /// Returns the closed renderer-neutral session state.
    #[must_use]
    pub const fn session_state(&self) -> SessionState {
        self.controller.state()
    }

    /// Returns the number of successfully completed logical actions.
    #[must_use]
    pub const fn completed_logical_steps(&self) -> u32 {
        self.controller.completed_logical_steps()
    }

    /// Returns the first uncaptured checkpoint declared at the current logical boundary.
    #[must_use]
    pub fn reachable_checkpoint_id(&self) -> Option<&CheckpointId> {
        let completed = self.controller.completed_logical_steps();
        let resolved = self.controller.selected()?;
        resolved
            .checkpoints()
            .iter()
            .find(|checkpoint| {
                checkpoint.logical_step() == completed
                    && !self.controller.captures().iter().any(|capture| {
                        capture.identity().checkpoint_id() == checkpoint.checkpoint_id()
                    })
            })
            .map(liquidfun_test_protocol::CheckpointDeclaration::checkpoint_id)
    }

    /// Returns the most recently captured owned canonical checkpoint.
    #[must_use]
    pub fn latest_checkpoint(&self) -> Option<&CanonicalCheckpoint> {
        self.controller
            .captures()
            .last()
            .map(liquidfun_differential::SessionCapture::value)
    }

    /// Submits one typed controller action through single-flight adapter admission.
    ///
    /// # Errors
    ///
    /// Returns a bounded adapter, command-counter, or controller failure.
    pub fn perform(&mut self, action: ControllerAction) -> Result<(), InteractiveTestbedError> {
        let maybe_replacement_selection = replacement_selection(&action);
        let resets_cadence = resets_cadence(&action);
        let command = self.adapter.begin(self.controller.state(), action)?;
        let Some(command_id) = self.controller.next_command_id() else {
            self.adapter.complete();
            return Err(InteractiveTestbedError::CommandCounterExhausted);
        };
        let result = self.controller.submit(command_id, command);
        self.adapter.complete();
        result?;
        if let Some(selection) = maybe_replacement_selection {
            self.maybe_current_selection = Some(selection);
        }
        if resets_cadence {
            self.accumulated_time = Duration::ZERO;
        }
        Ok(())
    }

    /// Enters explicit running state without advancing implicitly.
    ///
    /// # Errors
    ///
    /// Returns a bounded typed-action failure.
    pub fn run(&mut self) -> Result<(), InteractiveTestbedError> {
        self.perform(ControllerAction::Run)
    }

    /// Pauses automatic advancement without executing a logical action.
    ///
    /// # Errors
    ///
    /// Returns a bounded typed-action failure.
    pub fn pause(&mut self) -> Result<(), InteractiveTestbedError> {
        self.perform(ControllerAction::Pause)
    }

    /// Executes exactly one logical action and settles paused.
    ///
    /// # Errors
    ///
    /// Returns a bounded typed-action failure.
    pub fn step_once(&mut self) -> Result<(), InteractiveTestbedError> {
        self.perform(ControllerAction::StepOnce)
    }

    /// Reconstructs the selected native session from identical resolved bytes.
    ///
    /// # Errors
    ///
    /// Returns a bounded typed-action failure.
    pub fn restart(&mut self) -> Result<(), InteractiveTestbedError> {
        self.perform(ControllerAction::Restart)
    }

    /// Captures the first currently reachable uncaptured canonical checkpoint.
    ///
    /// # Errors
    ///
    /// Returns a bounded reachability, adapter, or controller failure.
    pub fn capture_reachable_checkpoint(&mut self) -> Result<(), InteractiveTestbedError> {
        let checkpoint_id = self
            .reachable_checkpoint_id()
            .ok_or(InteractiveTestbedError::NoReachableCheckpoint)?
            .clone();
        self.perform(ControllerAction::CaptureCheckpoint(checkpoint_id))
    }

    /// Re-resolves the selected exact catalog identity and restarts with new settings.
    ///
    /// # Errors
    ///
    /// Returns a bounded missing-selection, catalog, adapter, or controller failure.
    pub fn apply_settings(&mut self, settings: RunSettings) -> Result<(), InteractiveTestbedError> {
        let current = self
            .controller
            .selected()
            .ok_or(InteractiveTestbedError::NoSelection)?;
        let identity = current.identity();
        let request = ResolveRequest::new(identity.slug().clone(), identity.maybe_seed(), settings);
        let resolved = resolve_catalog(self.catalog.definitions(), &request)
            .map_err(|error| InteractiveTestbedError::Catalog(error.kind()))?;
        let input = RunSettingsInput::new(
            settings.timestep_bits(),
            settings.velocity_iterations(),
            settings.position_iterations(),
            settings.particle_iterations(),
        );
        self.perform(ControllerAction::ApplySettingsAndRestart {
            settings: input,
            resolved,
        })
    }

    /// Advances a running session from a fixed-time accumulator, never once per render frame.
    ///
    /// At most [`MAXIMUM_STEPS_PER_UPDATE`] logical actions execute per call. Excess elapsed time
    /// is discarded after the cap so a stalled renderer cannot create an unbounded catch-up loop.
    ///
    /// # Errors
    ///
    /// Returns a bounded controller failure from a logical action.
    pub fn update(&mut self, elapsed: Duration) -> Result<u32, InteractiveTestbedError> {
        if self.controller.state() != SessionState::Running {
            self.accumulated_time = Duration::ZERO;
            return Ok(0);
        }
        let Some(settings) = self.selected_settings() else {
            return Err(InteractiveTestbedError::NoSelection);
        };
        let timestep = Duration::from_secs_f64(f64::from(settings.timestep_bits().to_f32()));
        if timestep.is_zero() {
            self.accumulated_time = Duration::ZERO;
            return Err(InteractiveTestbedError::TimestepBelowClockResolution);
        }
        let maximum_accumulated = timestep.saturating_mul(MAXIMUM_STEPS_PER_UPDATE);
        self.accumulated_time = self
            .accumulated_time
            .saturating_add(elapsed)
            .min(maximum_accumulated);

        let mut completed = 0;
        while completed < MAXIMUM_STEPS_PER_UPDATE
            && self.accumulated_time >= timestep
            && self.controller.state() == SessionState::Running
        {
            self.controller.advance_running()?;
            self.accumulated_time = self.accumulated_time.saturating_sub(timestep);
            completed += 1;
        }
        if self.controller.state() != SessionState::Running {
            self.accumulated_time = Duration::ZERO;
        }
        Ok(completed)
    }

    fn resolve_default_selection(
        &self,
        selection: &CatalogSelection,
    ) -> Result<ResolvedScenario, InteractiveTestbedError> {
        let slug = CatalogSlug::new(selection.catalog_slug())
            .map_err(|error| InteractiveTestbedError::Catalog(error.kind()))?;
        let definition = self
            .catalog
            .definitions()
            .iter()
            .find(|definition| {
                definition.slug() == &slug
                    && definition.scenario_version().get() == selection.scenario_version()
            })
            .ok_or(InteractiveTestbedError::SelectionIdentityMismatch)?;
        let settings = definition
            .metadata()
            .ok_or(InteractiveTestbedError::SelectionIdentityMismatch)?
            .default_settings();
        let request = ResolveRequest::new(slug.clone(), selection.maybe_seed(), settings);
        let resolved = match resolve_catalog(self.catalog.definitions(), &request) {
            Ok(resolved) => resolved,
            Err(error)
                if error.kind() == CatalogErrorKind::SeedRequired
                    && selection.maybe_seed().is_none() =>
            {
                let seeded = ResolveRequest::new(slug, Some(DEFAULT_REQUIRED_SEED), settings);
                resolve_catalog(self.catalog.definitions(), &seeded)
                    .map_err(|error| InteractiveTestbedError::Catalog(error.kind()))?
            }
            Err(error) => return Err(InteractiveTestbedError::Catalog(error.kind())),
        };
        if resolved.identity().scenario_version().get() != selection.scenario_version() {
            return Err(InteractiveTestbedError::SelectionIdentityMismatch);
        }
        Ok(resolved)
    }
}

fn replacement_selection(action: &ControllerAction) -> Option<CatalogSelection> {
    let resolved = match action {
        ControllerAction::Select(resolved)
        | ControllerAction::ApplySettingsAndRestart { resolved, .. } => resolved,
        ControllerAction::Run
        | ControllerAction::Pause
        | ControllerAction::StepOnce
        | ControllerAction::Restart
        | ControllerAction::CaptureCheckpoint(_)
        | ControllerAction::ApplyScenarioAction(_) => return None,
    };
    let identity = resolved.identity();
    Some(CatalogSelection::new(
        identity.slug().as_str(),
        identity.scenario_version().get(),
        identity.maybe_seed(),
    ))
}

const fn resets_cadence(action: &ControllerAction) -> bool {
    !matches!(action, ControllerAction::CaptureCheckpoint(_))
}
