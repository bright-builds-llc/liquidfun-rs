use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::PathBuf;

const USAGE: &str = "Usage: cargo xtask docs check";
const TABLE_HEADING: &str = "## Testing layer contract";
const COLUMNS: [&str; 9] = [
    "Layer",
    "Status",
    "Purpose",
    "Command",
    "Prerequisites",
    "Reports and failure artifacts",
    "Retry policy",
    "Placement",
    "Semantic interpretation",
];
const ALLOWED_STATUSES: [&str; 2] = ["current", "deferred"];
const ALLOWED_PLACEMENTS: [&str; 4] = ["local", "pull request", "scheduled", "manual release"];
const PLACEHOLDERS: [&str; 5] = ["TODO", "TBD", "PLACEHOLDER", "REPLACE_ME", "REPLACE WITH"];

#[derive(Clone, Copy)]
struct LayerRule {
    name: &'static str,
    command: &'static str,
    prerequisite: &'static str,
    artifact: &'static str,
    retry: &'static str,
    semantics: &'static str,
}

const LAYER_RULES: [LayerRule; 12] = [
    LayerRule {
        name: "unit",
        command: "cargo test --workspace --lib",
        prerequisite: "Rust 1.97.0",
        artifact: "test output",
        retry: "No deterministic retry",
        semantics: "focused behavior",
    },
    LayerRule {
        name: "integration/API",
        command: "cargo test --workspace --tests",
        prerequisite: "Rust 1.97.0",
        artifact: "test output",
        retry: "No deterministic retry",
        semantics: "supported API",
    },
    LayerRule {
        name: "doctest",
        command: "cargo test --workspace --doc",
        prerequisite: "Rust 1.97.0",
        artifact: "rustdoc output",
        retry: "No deterministic retry",
        semantics: "documentation example",
    },
    LayerRule {
        name: "upstream compatibility",
        command: "cargo xtask upstream verify",
        prerequisite: "initialized submodule",
        artifact: "target/reference",
        retry: "No deterministic retry",
        semantics: "oracle infrastructure",
    },
    LayerRule {
        name: "differential",
        command: "cargo xtask differential compare",
        prerequisite: "oracle-debug",
        artifact: "target/differential/failures",
        retry: "No deterministic retry",
        semantics: "physics mismatch",
    },
    LayerRule {
        name: "property",
        command: "cargo test --workspace property",
        prerequisite: "Rust 1.97.0",
        artifact: "minimized input",
        retry: "No deterministic retry",
        semantics: "invariant",
    },
    LayerRule {
        name: "checked-in regression",
        command: "cargo xtask differential replay",
        prerequisite: "reviewed trace",
        artifact: "scenarios/regressions",
        retry: "No deterministic retry",
        semantics: "same failure signature",
    },
    LayerRule {
        name: "fuzz",
        command: "cargo fuzz run",
        prerequisite: "pinned nightly",
        artifact: "fuzz/artifacts",
        retry: "No deterministic retry",
        semantics: "harness failure",
    },
    LayerRule {
        name: "Miri/UB-aliasing",
        command: "cargo miri test",
        prerequisite: "pinned nightly",
        artifact: "Miri diagnostics",
        retry: "No deterministic retry",
        semantics: "harness failure",
    },
    LayerRule {
        name: "native sanitizer",
        command: "UBSAN_OPTIONS=halt_on_error=1:print_stacktrace=1",
        prerequisite: "Clang 22.1.8",
        artifact: "target/differential/failures",
        retry: "No deterministic retry",
        semantics: "harness failure",
    },
    LayerRule {
        name: "benchmark",
        command: "cargo bench --workspace",
        prerequisite: "controlled hardware",
        artifact: "target/criterion",
        retry: "No deterministic retry",
        semantics: "performance evidence, not parity",
    },
    LayerRule {
        name: "coverage",
        command: "cargo llvm-cov",
        prerequisite: "llvm-tools-preview",
        artifact: "target/llvm-cov",
        retry: "No deterministic retry",
        semantics: "coverage is not parity",
    },
];

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
    println!(
        "docs verified: {} testing layers with complete DOCS-05 contracts",
        LAYER_RULES.len()
    );
    Ok(())
}

fn check_testing_contract(contents: &str) -> Result<(), DocsError> {
    let table = parse_contract_table(contents)?;
    let mut rows = BTreeMap::new();
    for cells in table {
        validate_cells(&cells)?;
        let layer = cells[0].clone();
        if rows.insert(layer.clone(), cells).is_some() {
            return Err(DocsError::new(
                "layer",
                format!("duplicate testing layer `{layer}`"),
            ));
        }
    }

    if rows.len() != LAYER_RULES.len() {
        return Err(DocsError::new(
            "layer",
            format!(
                "testing contract must contain exactly {} layers, actual {}",
                LAYER_RULES.len(),
                rows.len()
            ),
        ));
    }

    for rule in LAYER_RULES {
        let Some(cells) = rows.get(rule.name) else {
            return Err(DocsError::new(
                "layer",
                format!("missing testing layer `{}`", rule.name),
            ));
        };
        validate_layer(rule, cells)?;
    }
    Ok(())
}

fn parse_contract_table(contents: &str) -> Result<Vec<Vec<String>>, DocsError> {
    let mut lines = contents.lines();
    let Some(_) = lines.find(|line| line.trim() == TABLE_HEADING) else {
        return Err(DocsError::new(
            "schema",
            format!("missing `{TABLE_HEADING}` heading"),
        ));
    };
    let Some(header) = lines.find(|line| line.trim_start().starts_with("| Layer |")) else {
        return Err(DocsError::new("schema", "missing testing table header"));
    };
    let header_cells = parse_row(header)?;
    if header_cells != COLUMNS {
        return Err(DocsError::new(
            "schema",
            format!(
                "testing table columns must be exactly: {}",
                COLUMNS.join(", ")
            ),
        ));
    }
    let Some(separator) = lines.next() else {
        return Err(DocsError::new("schema", "missing testing table separator"));
    };
    let separator_cells = parse_row(separator)?;
    if separator_cells.len() != COLUMNS.len()
        || separator_cells
            .iter()
            .any(|cell| cell.len() < 3 || !cell.chars().all(|character| character == '-'))
    {
        return Err(DocsError::new(
            "schema",
            "testing table separator must contain one dash group per column",
        ));
    }

    let mut rows = Vec::new();
    for line in lines {
        if !line.trim_start().starts_with('|') {
            break;
        }
        rows.push(parse_row(line)?);
    }
    if rows.is_empty() {
        return Err(DocsError::new("schema", "testing table has no rows"));
    }
    Ok(rows)
}

fn parse_row(line: &str) -> Result<Vec<String>, DocsError> {
    let trimmed = line.trim();
    let Some(contents) = trimmed
        .strip_prefix('|')
        .and_then(|contents| contents.strip_suffix('|'))
    else {
        return Err(DocsError::new(
            "schema",
            format!("invalid Markdown table row `{trimmed}`"),
        ));
    };
    let cells = contents
        .split('|')
        .map(|cell| cell.trim().to_owned())
        .collect::<Vec<_>>();
    if cells.len() != COLUMNS.len() {
        return Err(DocsError::new(
            "schema",
            format!(
                "testing row must contain {} cells, actual {}",
                COLUMNS.len(),
                cells.len()
            ),
        ));
    }
    Ok(cells)
}

fn validate_cells(cells: &[String]) -> Result<(), DocsError> {
    for (index, cell) in cells.iter().enumerate() {
        if cell.trim().is_empty() {
            return Err(DocsError::new(
                "schema",
                format!("testing row column `{}` cannot be blank", COLUMNS[index]),
            ));
        }
        let uppercase = cell.to_ascii_uppercase();
        if let Some(placeholder) = PLACEHOLDERS
            .iter()
            .find(|placeholder| uppercase.contains(*placeholder))
        {
            return Err(DocsError::new(
                "placeholder",
                format!(
                    "testing row column `{}` contains forbidden placeholder `{placeholder}`",
                    COLUMNS[index]
                ),
            ));
        }
    }
    Ok(())
}

fn validate_layer(rule: LayerRule, cells: &[String]) -> Result<(), DocsError> {
    require_allowed(&cells[1], &ALLOWED_STATUSES, "status", rule.name)?;
    let placements = cells[7].split(',').map(str::trim).collect::<Vec<_>>();
    for placement in placements {
        require_allowed(placement, &ALLOWED_PLACEMENTS, "placement", rule.name)?;
    }

    for (column, value, marker) in [
        ("Command", cells[3].as_str(), rule.command),
        ("Prerequisites", cells[4].as_str(), rule.prerequisite),
        (
            "Reports and failure artifacts",
            cells[5].as_str(),
            rule.artifact,
        ),
        ("Retry policy", cells[6].as_str(), rule.retry),
        ("Semantic interpretation", cells[8].as_str(), rule.semantics),
    ] {
        if !value
            .to_ascii_lowercase()
            .contains(&marker.to_ascii_lowercase())
        {
            return Err(DocsError::new(
                "content",
                format!(
                    "testing layer `{}` column `{column}` must contain `{marker}`",
                    rule.name
                ),
            ));
        }
    }
    Ok(())
}

fn require_allowed(
    value: &str,
    allowed: &[&str],
    field: &'static str,
    layer: &str,
) -> Result<(), DocsError> {
    if allowed.contains(&value) {
        return Ok(());
    }
    Err(DocsError::new(
        field,
        format!(
            "testing layer `{layer}` has invalid {field} `{value}`; allowed values: {}",
            allowed.join(", ")
        ),
    ))
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
