use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::upstream::UpstreamError;

use super::USAGE;

/// Closed error returned by exploratory playground timing commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlaygroundError {
    kind: &'static str,
    message: String,
}

impl PlaygroundError {
    pub(crate) fn new(kind: &'static str, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub(crate) fn usage(message: impl Into<String>) -> Self {
        Self::new("usage", format!("{}\n\n{USAGE}", message.into()))
    }
}

impl Display for PlaygroundError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "playground/{}: {}", self.kind, self.message)
    }
}

impl Error for PlaygroundError {}

impl From<UpstreamError> for PlaygroundError {
    fn from(error: UpstreamError) -> Self {
        Self::new("upstream", error.to_string())
    }
}
