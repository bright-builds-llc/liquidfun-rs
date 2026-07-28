use std::env;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::PathBuf;

mod contracts;

use contracts::validation::{check_document_contracts, check_testing_contract};
#[allow(
    clippy::wildcard_imports,
    reason = "this split module shares its parent private contract"
)]
use contracts::*;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DocsError {
    category: &'static str,
    message: String,
}

impl DocsError {
    fn new(category: &'static str, message: impl Into<String>) -> Self {
        Self {
            category,
            message: message.into(),
        }
    }

    fn usage(message: impl Into<String>) -> Self {
        Self::new("usage", format!("{}\n\n{USAGE}", message.into()))
    }
}

impl Display for DocsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "docs/{}: {}", self.category, self.message)
    }
}

impl Error for DocsError {}

pub(crate) fn run(args: &[String]) -> Result<(), DocsError> {
    if args != ["check"] {
        return Err(DocsError::usage("expected `check`"));
    }
    let repository_root = repository_root()?;
    let testing_path = repository_root.join("TESTING.md");
    let contents = fs::read_to_string(&testing_path).map_err(|error| {
        DocsError::new(
            "filesystem",
            format!("failed to read {}: {error}", testing_path.display()),
        )
    })?;
    check_testing_contract(&contents)?;
    check_document_contracts(&repository_root)?;
    println!(
        "docs verified: {} testing layers, {} Phase 4, {} Phase 5, {} Phase 6, {} Phase 7, {} Phase 8, and {} Phase 12 public document contracts",
        LAYER_RULES.len(),
        DOCUMENT_CONTRACTS.len(),
        PHASE5_DOCUMENT_CONTRACTS.len(),
        PHASE6_DOCUMENT_CONTRACTS.len(),
        PHASE7_DOCUMENT_CONTRACTS.len(),
        PHASE8_DOCUMENT_CONTRACTS.len(),
        PHASE12_PUBLIC_DOCUMENT_CONTRACTS.len()
    );
    Ok(())
}

fn repository_root() -> Result<PathBuf, DocsError> {
    if let Some(root) = env::var_os("LIQUIDFUN_XTASK_ROOT") {
        return Ok(PathBuf::from(root));
    }
    let current_dir = env::current_dir().map_err(|error| {
        DocsError::new(
            "filesystem",
            format!("failed to read current directory: {error}"),
        )
    })?;
    let Some(root) = current_dir.ancestors().find(|candidate| {
        candidate.join("TESTING.md").is_file() && candidate.join("Cargo.toml").is_file()
    }) else {
        return Err(DocsError::new(
            "repository",
            "could not find TESTING.md and Cargo.toml",
        ));
    };
    Ok(root.to_path_buf())
}
