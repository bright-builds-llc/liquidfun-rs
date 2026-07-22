//! Inspector tabs and exact operational/error presentation.

const MAXIMUM_ERROR_FIELD_BYTES: usize = 512;

/// Four approved inspector tabs in keyboard order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorTab {
    Run,
    Observe,
    Differences,
    Provenance,
}

impl InspectorTab {
    /// Stable visual and keyboard order.
    pub const ALL: [Self; 4] = [
        Self::Run,
        Self::Observe,
        Self::Differences,
        Self::Provenance,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Run => "Run",
            Self::Observe => "Observe",
            Self::Differences => "Differences",
            Self::Provenance => "Provenance",
        }
    }
}

/// Closed inspector state separate from comparison authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorState {
    NoSelection,
    Resolving,
    Comparing,
    ExactMatch,
    RecoverableError,
    HarnessFailure,
    OracleUnavailable,
}

/// Exact heading/body copy for one state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperationalCopy {
    heading: &'static str,
    body: &'static str,
}

impl OperationalCopy {
    #[must_use]
    pub const fn heading(self) -> &'static str {
        self.heading
    }

    #[must_use]
    pub const fn body(self) -> &'static str {
        self.body
    }
}

/// Returns exact UI-SPEC state copy.
#[must_use]
pub const fn operational_copy(state: InspectorState) -> OperationalCopy {
    match state {
        InspectorState::NoSelection => OperationalCopy {
            heading: "Select a scenario",
            body: "Choose a reviewed catalog scenario to resolve its run plan and inspect it headlessly or visually.",
        },
        InspectorState::Resolving => OperationalCopy {
            heading: "Resolving scenario…",
            body: "Previous run",
        },
        InspectorState::Comparing => OperationalCopy {
            heading: "Comparing semantic checkpoints…",
            body: "The last valid checkpoint remains visible.",
        },
        InspectorState::ExactMatch => OperationalCopy {
            heading: "No differences at this checkpoint",
            body: "Rust and oracle observations match under the selected policies.",
        },
        InspectorState::RecoverableError => OperationalCopy {
            heading: "Scenario could not start",
            body: "Review the run details, correct the issue, and try again.",
        },
        InspectorState::HarnessFailure => OperationalCopy {
            heading: "Harness failure",
            body: "Review bounded details, configure the backend, and retry.",
        },
        InspectorState::OracleUnavailable => OperationalCopy {
            heading: "Oracle unavailable",
            body: "Oracle unavailable. Continue with Rust-only diagnostics or configure the pinned oracle.",
        },
    }
}

/// Bounded recoverable or harness error retaining last valid checkpoint identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorPanel {
    state: InspectorState,
    problem: Box<str>,
    recovery: Box<str>,
    details: Box<str>,
    maybe_retained_checkpoint: Option<Box<str>>,
}

impl ErrorPanel {
    /// Parses bounded, non-sensitive presentation fields.
    ///
    /// # Errors
    ///
    /// Rejects non-error states, empty fields, controls, or oversized values.
    pub fn new(
        state: InspectorState,
        problem: &str,
        recovery: &str,
        details: &str,
        maybe_retained_checkpoint: Option<&str>,
    ) -> Result<Self, InspectorError> {
        if !matches!(
            state,
            InspectorState::RecoverableError | InspectorState::HarnessFailure
        ) {
            return Err(InspectorError);
        }
        for value in [problem, recovery, details] {
            validate_bounded(value)?;
        }
        if let Some(checkpoint) = maybe_retained_checkpoint {
            validate_bounded(checkpoint)?;
        }
        Ok(Self {
            state,
            problem: problem.into(),
            recovery: recovery.into(),
            details: details.into(),
            maybe_retained_checkpoint: maybe_retained_checkpoint.map(Into::into),
        })
    }

    #[must_use]
    pub const fn heading(&self) -> &'static str {
        operational_copy(self.state).heading()
    }

    #[must_use]
    pub fn problem(&self) -> &str {
        &self.problem
    }

    #[must_use]
    pub fn recovery(&self) -> &str {
        &self.recovery
    }

    #[must_use]
    pub fn details(&self) -> &str {
        &self.details
    }

    #[must_use]
    pub fn retained_checkpoint(&self) -> Option<&str> {
        self.maybe_retained_checkpoint.as_deref()
    }
}

fn validate_bounded(value: &str) -> Result<(), InspectorError> {
    if value.is_empty()
        || value.len() > MAXIMUM_ERROR_FIELD_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(InspectorError);
    }
    Ok(())
}

/// Bounded inspector presentation error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("inspector presentation input is invalid")]
pub struct InspectorError;
