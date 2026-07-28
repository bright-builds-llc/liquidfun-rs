use crate::BodyId;
use crate::collision::{CollisionError, TimeOfImpactInput, TimeOfImpactState, time_of_impact};
use crate::math::settings::{EPSILON, MAX_SUB_STEPS};
use crate::math::{SweepError, min};

use super::config::StepConfiguration;
use super::contact::{ToiAlpha, ToiCountLimitReached};
use super::contact_manager::ContactManager;
use super::contact_solver::{ContactSolve, ContactSolveFailure};
use super::fixture::FixtureBoundsError;
use super::island::{
    IslandBuildError, ToiIsland, ToiIslandLimits, ToiIslandSolution, solve_toi_island,
};
use super::object::World;
use super::proxy::PreparedSynchronization;
use super::step::{CollisionDecisionHook, ContactHookRun, StepError};

const REVIEWED_MAX_CCD_SCAN_CONTACTS: usize = 8_192;

mod event;
mod step_state;
pub(super) use step_state::{ContinuousStepKey, ContinuousStepKind, ContinuousStepState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ContinuousContactIndex(usize);

impl ContinuousContactIndex {
    fn new(index: usize, contact_count: usize) -> Result<Self, ContinuousScanError> {
        if index >= contact_count || index >= REVIEWED_MAX_CCD_SCAN_CONTACTS {
            return Err(ContinuousScanError::InvalidGraph);
        }
        Ok(Self(index))
    }

    const fn get(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ContinuousCandidate {
    contact_index: ContinuousContactIndex,
    alpha: ToiAlpha,
    bodies: [BodyId; 2],
}

impl ContinuousCandidate {
    pub(super) const fn contact_index(self) -> usize {
        self.contact_index.get()
    }

    pub(super) const fn alpha(self) -> f32 {
        self.alpha.get()
    }

    pub(super) const fn bodies(self) -> [BodyId; 2] {
        self.bodies
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct ContinuousScanControl {
    maybe_reject_ordinal: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum ContinuousScanError {
    CapacityExceeded {
        resource: &'static str,
        limit: usize,
    },
    InvalidGraph,
    Collision(CollisionError),
    Sweep(SweepError),
    ToiCountLimit,
    Hook(StepError),
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum ContinuousEventError {
    Scan(ContinuousScanError),
    Island(IslandBuildError),
    Solve(ContactSolveFailure),
    ProxyBounds,
    InjectedFailure,
}

impl From<ContinuousScanError> for ContinuousEventError {
    fn from(error: ContinuousScanError) -> Self {
        Self::Scan(error)
    }
}

impl From<IslandBuildError> for ContinuousEventError {
    fn from(error: IslandBuildError) -> Self {
        Self::Island(error)
    }
}

impl From<ContactSolveFailure> for ContinuousEventError {
    fn from(error: ContactSolveFailure) -> Self {
        Self::Solve(error)
    }
}

impl From<FixtureBoundsError> for ContinuousEventError {
    fn from(_error: FixtureBoundsError) -> Self {
        Self::ProxyBounds
    }
}

impl From<SweepError> for ContinuousEventError {
    fn from(error: SweepError) -> Self {
        Self::Scan(ContinuousScanError::Sweep(error))
    }
}

#[derive(Debug)]
pub(super) struct ContinuousEvent {
    #[cfg(feature = "differential-internals")]
    pub(super) body_ids: Vec<BodyId>,
    #[cfg(feature = "differential-internals")]
    pub(super) contact_occurrences: Vec<u64>,
    #[cfg(feature = "differential-internals")]
    pub(super) transient_normal_impulse_sum: f32,
    pub(super) contact_solves: Vec<ContactSolve>,
}

struct ContinuousWorldBackup {
    bodies: Vec<(BodyId, super::body::BodyState, bool, bool)>,
    contact_manager: ContactManager,
}

impl From<CollisionError> for ContinuousScanError {
    fn from(error: CollisionError) -> Self {
        Self::Collision(error)
    }
}

impl From<SweepError> for ContinuousScanError {
    fn from(error: SweepError) -> Self {
        Self::Sweep(error)
    }
}

impl From<ToiCountLimitReached> for ContinuousScanError {
    fn from(_error: ToiCountLimitReached) -> Self {
        Self::ToiCountLimit
    }
}

impl World {
    #[cfg(any(test, feature = "differential-internals"))]
    fn select_continuous_candidate_with_control(
        &mut self,
        control: ContinuousScanControl,
    ) -> Result<Option<ContinuousCandidate>, ContinuousScanError> {
        let mut hook = super::step::NoDecisionHook;
        let mut hook_run = ContactHookRun::new(&mut hook, super::step::StepLimits::default());
        self.select_continuous_candidate_with_hook(control, &mut hook_run)
    }

    fn select_continuous_candidate_with_hook<H: CollisionDecisionHook>(
        &mut self,
        control: ContinuousScanControl,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<Option<ContinuousCandidate>, ContinuousScanError> {
        let contact_count = self.contact_manager.len();
        if contact_count > REVIEWED_MAX_CCD_SCAN_CONTACTS {
            return Err(ContinuousScanError::CapacityExceeded {
                resource: "ccd scan contacts",
                limit: REVIEWED_MAX_CCD_SCAN_CONTACTS,
            });
        }

        let mut rejection_count = 0_usize;
        loop {
            let Some(candidate) = self.scan_earliest_continuous_contact()? else {
                return Ok(None);
            };
            if self.validate_continuous_candidate(candidate, control, hook_run)? {
                return Ok(Some(candidate));
            }
            rejection_count =
                rejection_count
                    .checked_add(1)
                    .ok_or(ContinuousScanError::CapacityExceeded {
                        resource: "ccd rejected candidates",
                        limit: REVIEWED_MAX_CCD_SCAN_CONTACTS,
                    })?;
            if rejection_count > contact_count {
                return Err(ContinuousScanError::InvalidGraph);
            }
        }
    }

    fn scan_earliest_continuous_contact(
        &mut self,
    ) -> Result<Option<ContinuousCandidate>, ContinuousScanError> {
        let contact_count = self.contact_manager.len();
        let mut maybe_candidate = None;
        let mut minimum_alpha = 1.0_f32;

        for index in 0..contact_count {
            let maybe_alpha = self.continuous_contact_alpha(index)?;
            let Some(alpha) = maybe_alpha else {
                continue;
            };
            if alpha.get() < minimum_alpha {
                let contact = self
                    .contact_manager
                    .contacts()
                    .get(index)
                    .ok_or(ContinuousScanError::InvalidGraph)?;
                maybe_candidate = Some(ContinuousCandidate {
                    contact_index: ContinuousContactIndex::new(index, contact_count)?,
                    alpha,
                    bodies: [contact.key.first.body, contact.key.second.body],
                });
                minimum_alpha = alpha.get();
            }
        }

        if 1.0 - 10.0 * EPSILON < minimum_alpha {
            return Ok(None);
        }
        Ok(maybe_candidate)
    }

    fn continuous_contact_alpha(
        &mut self,
        index: usize,
    ) -> Result<Option<ToiAlpha>, ContinuousScanError> {
        let contact = self
            .contact_manager
            .contacts()
            .get(index)
            .ok_or(ContinuousScanError::InvalidGraph)?;
        if !contact.is_enabled() || contact.toi_count() > MAX_SUB_STEPS {
            return Ok(None);
        }
        if let Some(alpha) = contact.maybe_toi_alpha() {
            return Ok(Some(alpha));
        }

        let key = contact.key;
        let fixture_a = self
            .fixtures
            .get(key.first.fixture)
            .map_err(|_error| ContinuousScanError::InvalidGraph)?;
        let fixture_b = self
            .fixtures
            .get(key.second.fixture)
            .map_err(|_error| ContinuousScanError::InvalidGraph)?;
        if fixture_a.definition.is_sensor() || fixture_b.definition.is_sensor() {
            return Ok(None);
        }
        let body_a = self
            .bodies
            .get(key.first.body)
            .map_err(|_error| ContinuousScanError::InvalidGraph)?;
        let body_b = self
            .bodies
            .get(key.second.body)
            .map_err(|_error| ContinuousScanError::InvalidGraph)?;
        let snapshot_a = body_a.state.snapshot();
        let snapshot_b = body_b.state.snapshot();
        let active_a =
            snapshot_a.is_awake() && snapshot_a.body_type() != super::body::BodyType::Static;
        let active_b =
            snapshot_b.is_awake() && snapshot_b.body_type() != super::body::BodyType::Static;
        if !active_a && !active_b {
            return Ok(None);
        }
        let collide_a =
            snapshot_a.is_bullet() || snapshot_a.body_type() != super::body::BodyType::Dynamic;
        let collide_b =
            snapshot_b.is_bullet() || snapshot_b.body_type() != super::body::BodyType::Dynamic;
        if !collide_a && !collide_b {
            return Ok(None);
        }

        let (sweep_a, sweep_b, alpha0) =
            self.equalized_contact_sweeps(key.first.body, key.second.body)?;
        let alpha = if alpha0 >= 1.0 {
            ToiAlpha::new(1.0).ok_or(ContinuousScanError::InvalidGraph)?
        } else {
            let fixture_a = self
                .fixtures
                .get(key.first.fixture)
                .map_err(|_error| ContinuousScanError::InvalidGraph)?;
            let fixture_b = self
                .fixtures
                .get(key.second.fixture)
                .map_err(|_error| ContinuousScanError::InvalidGraph)?;
            let input = TimeOfImpactInput::new(
                fixture_a.definition.shape(),
                key.first.child_index,
                sweep_a,
                fixture_b.definition.shape(),
                key.second.child_index,
                sweep_b,
                1.0,
            )?;
            let output = time_of_impact(&input)?;
            let value = if output.state() == TimeOfImpactState::Touching {
                min(alpha0 + (1.0 - alpha0) * output.time(), 1.0)
            } else {
                1.0
            };
            ToiAlpha::new(value).ok_or(ContinuousScanError::InvalidGraph)?
        };
        self.contact_manager
            .contact_mut(index)
            .ok_or(ContinuousScanError::InvalidGraph)?
            .cache_toi_alpha(alpha);
        Ok(Some(alpha))
    }

    fn equalized_contact_sweeps(
        &mut self,
        body_a: BodyId,
        body_b: BodyId,
    ) -> Result<(crate::math::Sweep, crate::math::Sweep, f32), ContinuousScanError> {
        if body_a == body_b {
            return Err(ContinuousScanError::InvalidGraph);
        }
        let mut state_a = self
            .bodies
            .get(body_a)
            .map_err(|_error| ContinuousScanError::InvalidGraph)?
            .state;
        let mut state_b = self
            .bodies
            .get(body_b)
            .map_err(|_error| ContinuousScanError::InvalidGraph)?
            .state;
        let mut alpha0 = state_a.sweep().initial_fraction();
        if state_a.sweep().initial_fraction() < state_b.sweep().initial_fraction() {
            alpha0 = state_b.sweep().initial_fraction();
            state_a = state_a.candidate_equalize_sweep(alpha0)?;
            self.bodies
                .get_mut(body_a)
                .map_err(|_error| ContinuousScanError::InvalidGraph)?
                .state = state_a;
        } else if state_b.sweep().initial_fraction() < state_a.sweep().initial_fraction() {
            alpha0 = state_a.sweep().initial_fraction();
            state_b = state_b.candidate_equalize_sweep(alpha0)?;
            self.bodies
                .get_mut(body_b)
                .map_err(|_error| ContinuousScanError::InvalidGraph)?
                .state = state_b;
        }
        Ok((state_a.sweep(), state_b.sweep(), alpha0))
    }

    fn validate_continuous_candidate<H: CollisionDecisionHook>(
        &mut self,
        candidate: ContinuousCandidate,
        control: ContinuousScanControl,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<bool, ContinuousScanError> {
        let index = candidate.contact_index();
        let contact = self
            .contact_manager
            .contacts()
            .get(index)
            .ok_or(ContinuousScanError::InvalidGraph)?;
        let ordinal = contact.ordinal;
        let [body_a, body_b] = candidate.bodies();
        if body_a == body_b {
            return Err(ContinuousScanError::InvalidGraph);
        }
        let backup_a = self
            .bodies
            .get(body_a)
            .map_err(|_error| ContinuousScanError::InvalidGraph)?
            .state;
        let backup_b = self
            .bodies
            .get(body_b)
            .map_err(|_error| ContinuousScanError::InvalidGraph)?
            .state;
        let advanced_a = backup_a.candidate_advance_to(candidate.alpha())?;
        let advanced_b = backup_b.candidate_advance_to(candidate.alpha())?;
        self.bodies
            .get_mut(body_a)
            .map_err(|_error| ContinuousScanError::InvalidGraph)?
            .state = advanced_a;
        self.bodies
            .get_mut(body_b)
            .map_err(|_error| ContinuousScanError::InvalidGraph)?
            .state = advanced_b;

        self.contact_manager
            .refresh_continuous_contact_with_hook(index, &mut self.bodies, &self.fixtures, hook_run)
            .map_err(ContinuousScanError::Hook)?
            .ok_or(ContinuousScanError::InvalidGraph)?;
        let contact = self
            .contact_manager
            .contact_mut(index)
            .ok_or(ContinuousScanError::InvalidGraph)?;
        contact.increment_toi_count()?;
        if control.maybe_reject_ordinal == Some(ordinal) {
            contact.set_enabled(false);
        }
        let accepted = contact.is_enabled() && contact.is_touching();
        if !accepted {
            contact.set_enabled(false);
            self.bodies
                .get_mut(body_a)
                .map_err(|_error| ContinuousScanError::InvalidGraph)?
                .state = backup_a;
            self.bodies
                .get_mut(body_b)
                .map_err(|_error| ContinuousScanError::InvalidGraph)?
                .state = backup_b;
            return Ok(false);
        }

        for body_id in [body_a, body_b] {
            let body = self
                .bodies
                .get_mut(body_id)
                .map_err(|_error| ContinuousScanError::InvalidGraph)?;
            body.state = body.state.candidate_set_awake(true);
            body.pending_wake = false;
        }
        Ok(true)
    }

    #[cfg(feature = "differential-internals")]
    #[doc(hidden)]
    pub fn rigid_toi_event_diagnostic(
        &mut self,
        configuration: StepConfiguration,
        limits: crate::rigid_differential::RigidToiIslandLimits,
        maybe_failure: Option<crate::rigid_differential::RigidToiFailureInjection>,
    ) -> Result<
        Option<crate::rigid_differential::RigidToiEventDiagnostic>,
        crate::rigid_differential::RigidToiSolveError,
    > {
        let reviewed = ToiIslandLimits::REVIEWED;
        if limits.max_bodies() > reviewed.max_bodies
            || limits.max_contacts() > reviewed.max_contacts
        {
            return Err(
                crate::rigid_differential::RigidToiSolveError::CapacityExceeded {
                    resource: "TOI island diagnostic limits",
                    limit: reviewed.max_bodies.max(reviewed.max_contacts),
                },
            );
        }
        let inject_after_solve = matches!(
            maybe_failure,
            Some(crate::rigid_differential::RigidToiFailureInjection::AfterSolve)
        );
        self.solve_next_continuous_event(
            configuration,
            ToiIslandLimits {
                max_bodies: limits.max_bodies(),
                max_contacts: limits.max_contacts(),
            },
            inject_after_solve,
        )
        .map(|maybe_event| {
            maybe_event.map(|event| {
                crate::rigid_differential::RigidToiEventDiagnostic::new(
                    event.body_ids,
                    event.contact_occurrences,
                    event.transient_normal_impulse_sum,
                )
            })
        })
        .map_err(|error| rigid_toi_solve_error(&error))
    }

    #[cfg(feature = "differential-internals")]
    #[doc(hidden)]
    pub fn rigid_ccd_candidate_diagnostic(
        &mut self,
        maybe_injection: Option<crate::rigid_differential::RigidCcdFailureInjection>,
    ) -> Result<
        Option<crate::rigid_differential::RigidCcdCandidateDiagnostic>,
        crate::rigid_differential::RigidCcdScanError,
    > {
        let mut control = ContinuousScanControl::default();
        if let Some(injection) = maybe_injection {
            let (occurrence, reject) = match injection {
                crate::rigid_differential::RigidCcdFailureInjection::RejectCandidate {
                    occurrence,
                } => (occurrence, true),
                crate::rigid_differential::RigidCcdFailureInjection::ExhaustSubStepBudget {
                    occurrence,
                } => (occurrence, false),
            };
            let ordinal = occurrence
                .checked_sub(1)
                .ok_or(crate::rigid_differential::RigidCcdScanError::InvalidState)?;
            if reject {
                if self
                    .contact_manager
                    .contact_index_for_ordinal(ordinal)
                    .is_none()
                {
                    return Err(crate::rigid_differential::RigidCcdScanError::InvalidState);
                }
                control.maybe_reject_ordinal = Some(ordinal);
            } else {
                self.contact_manager
                    .exhaust_toi_budget_for_diagnostic(ordinal)
                    .ok_or(crate::rigid_differential::RigidCcdScanError::InvalidState)?;
            }
        }

        let maybe_candidate = self
            .select_continuous_candidate_with_control(control)
            .map_err(|error| match error {
                ContinuousScanError::CapacityExceeded { resource, limit } => {
                    crate::rigid_differential::RigidCcdScanError::CapacityExceeded {
                        resource,
                        limit,
                    }
                }
                ContinuousScanError::InvalidGraph
                | ContinuousScanError::Collision(_)
                | ContinuousScanError::Sweep(_)
                | ContinuousScanError::ToiCountLimit
                | ContinuousScanError::Hook(_) => {
                    crate::rigid_differential::RigidCcdScanError::InvalidState
                }
            })?;
        maybe_candidate
            .map(|candidate| {
                let contact = self
                    .contact_manager
                    .contacts()
                    .get(candidate.contact_index())
                    .ok_or(crate::rigid_differential::RigidCcdScanError::InvalidState)?;
                Ok(crate::rigid_differential::RigidCcdCandidateDiagnostic::new(
                    contact.ordinal + 1,
                    candidate.alpha(),
                    contact.snapshot(),
                ))
            })
            .transpose()
    }
}

#[cfg(feature = "differential-internals")]
fn rigid_toi_solve_error(
    error: &ContinuousEventError,
) -> crate::rigid_differential::RigidToiSolveError {
    use crate::rigid_differential::RigidToiSolveError;

    match error {
        ContinuousEventError::Island(IslandBuildError::CapacityExceeded { resource, limit })
        | ContinuousEventError::Solve(ContactSolveFailure::CapacityExceeded { resource, limit })
        | ContinuousEventError::Scan(ContinuousScanError::CapacityExceeded { resource, limit }) => {
            RigidToiSolveError::CapacityExceeded {
                resource,
                limit: *limit,
            }
        }
        ContinuousEventError::InjectedFailure => RigidToiSolveError::InjectedFailure,
        ContinuousEventError::Scan(
            ContinuousScanError::InvalidGraph
            | ContinuousScanError::Collision(_)
            | ContinuousScanError::Sweep(_)
            | ContinuousScanError::ToiCountLimit
            | ContinuousScanError::Hook(_),
        )
        | ContinuousEventError::Island(IslandBuildError::InvalidGraph)
        | ContinuousEventError::Solve(
            ContactSolveFailure::UnsupportedTopology
            | ContactSolveFailure::NonFinite
            | ContactSolveFailure::InvalidProxyBounds,
        )
        | ContinuousEventError::ProxyBounds => RigidToiSolveError::InvalidState,
    }
}

#[cfg(test)]
mod tests;
