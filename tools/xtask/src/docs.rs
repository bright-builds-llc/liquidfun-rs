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
const DOCUMENT_CONTRACTS: [(&str, &[&str]); 4] = [
    (
        "ARCHITECTURE.md",
        &[
            "## Phase 4 math and numerical boundaries",
            "`Vec2`, `Vec3`, `Vec4`, `Mat22`, `Mat33`, `Rotation`, `Transform`, and `Sweep`",
            "(MKS), angles use radians",
            "operations are column-major",
            "external C++ adapter",
            "None is a dependency or feature of `liquidfun`",
        ],
    ),
    (
        "TESTING.md",
        &[
            "## Phase 4 numerical policy",
            "The four float policies are",
            "`ExactBits`",
            "`Ulps`",
            "`Absolute`",
            "`AbsoluteRelative`",
            "Arithmetic NaN is a mismatch",
            "Positive and negative zero are distinct by default",
            "`Operation`",
            "`PhaseLocal`",
            "`ScenarioSteps(n)`",
            "D1 is canonical parity",
            "promote canonical fixtures.",
            "D2 cannot promote canonical fixtures",
            "## Phase 4 math-probe commands",
            "cargo xtask differential compare --scenario math-probes --preset oracle-debug --session-profile one-shot",
            "cargo xtask differential compare --scenario math-probes --preset oracle-release --session-profile one-shot",
            "cargo xtask differential replay --scenario math-probes --preset oracle-debug --session-profile one-shot",
            "cargo xtask differential verify-determinism --scenario math-probes --preset oracle-debug --runs 2",
        ],
    ),
    (
        "COMPATIBILITY.md",
        &[
            "| `implemented` | 30 | 147 |",
            "| `unit_tested` | 30 | 147 |",
            "| `differentially_validated` | 29 | 148 |",
            "| `platform_validated` | 0 | 177 |",
            "| `documented_difference` | 30 | 147 |",
            "`subsystem.common-math-and-settings`",
            "`public-api.liquidfun-box2d-box2d-common-b2math-h`",
            "`public-api.liquidfun-box2d-box2d-common-b2settings-h`",
            "| applicable | yes | yes | yes | yes | no | no | yes | no |",
        ],
    ),
    (
        "README.md",
        &[
            "bounded Phase 4 math",
            "canonical-platform evidence, performance, and production maturity",
            "remain pending",
        ],
    ),
];
const PHASE5_DOCUMENT_CONTRACTS: [(&str, &[&str]); 4] = [
    (
        "ARCHITECTURE.md",
        &[
            "## Phase 5 collision boundaries",
            "`CircleShape`, `EdgeShape`, `PolygonShape`, and `ChainShape`",
            "`[0.5, 2.0]`",
            "`distance < 10.0 * EPSILON`",
            "Collect-all query and ray membership is set-like",
            "The `differential-internals` feature is non-default, `#[doc(hidden)]`",
            "the Phase 5 portion of `COLL-05`",
            "contact persistence or",
            "Phase 6",
            "Bright Builds architecture",
        ],
    ),
    (
        "TESTING.md",
        &[
            "## Phase 5 collision comparison policy",
            "Exact `u32` bit transport",
            "`ExactBits` policies",
            "`Operation`",
            "`PhaseLocal`",
            "D0 requires two byte-identical",
            "D1 requires the canonical pinned Rust 1.97.0 and Clang 22.1.8",
            "D2-scoped",
            "78-case `collision-probes`",
            "world contact lifecycle remains",
            "## Phase 5 collision-probe commands",
            "cargo xtask differential compare --scenario collision-probes --preset oracle-debug --session-profile one-shot",
            "cargo xtask differential compare --scenario collision-probes --preset oracle-release --session-profile one-shot",
            "cargo xtask differential replay --scenario collision-probes --preset oracle-debug --session-profile one-shot",
            "cargo xtask differential verify-determinism --scenario collision-probes --preset oracle-debug --runs 2",
        ],
    ),
    (
        "COMPATIBILITY.md",
        &[
            "| `subsystem.collision-broad-phase` | `liquidfun/Box2D/Box2D/Collision` | `liquidfun::collision` | applicable | yes | yes | yes | yes | yes | no | yes | no |",
            "| `subsystem.collision-distance-and-toi` | `liquidfun/Box2D/Box2D/Collision` | `liquidfun::collision` | applicable | yes | yes | yes | yes | yes | no | yes | no |",
            "| `subsystem.collision-shapes-and-manifolds` | `liquidfun/Box2D/Box2D/Collision/Shapes` | `liquidfun::collision` | applicable | yes | yes | yes | yes | yes | no | yes | no |",
        ],
    ),
    (
        "README.md",
        &[
            "Phase 5 immutable shape/collision substrate",
            "78-case Phase 5 collision corpora",
            "world contact lifecycle",
            "remain pending",
        ],
    ),
];
const PHASE6_DOCUMENT_CONTRACTS: [(&str, &[&str]); 5] = [
    (
        "ARCHITECTURE.md",
        &[
            "## Phase 6 rigid-world boundaries",
            "`BodyDef` and `FixtureDef`",
            "`World::set_body_transform`",
            "`World::reset_body_mass_data`",
            "`World::set_body_mass_data`",
            "`World::set_fixture_sensor`",
            "`World::set_fixture_filter`",
            "creation-time mixed friction and restitution",
            "`FindPairs`, `UpdateContacts`, `Hook`, `Solve`, `Unlock`",
            "one static/dynamic contact",
            "Aggregate mass is a validate-before-commit transaction",
            "Admission requires at least one dynamic body",
            "timestep bits `0x3c888889`, eight",
            "at most 128 actions",
            "finite non-negative centered inertia",
            "authority are checked before every candidate",
            "`oracle-asan-ubsan` lane executes the C++ protocol target",
            "No durable contact identity",
            "Phase 7",
            "Phase 8",
        ],
    ),
    (
        "TESTING.md",
        &[
            "## Phase 6 rigid-world comparison policy",
            "`phase6-v1`",
            "`non_colliding_body_fixture_lifecycle`",
            "`single_contact_lifecycle`",
            "declaration-first",
            "Local successful comparisons are D2",
            "D0 requires exactly two byte-identical",
            "D1 remains the only fixture-promotion authority",
            "## Phase 6 rigid-world commands",
            "cargo xtask differential compare --scenario rigid-world --preset oracle-debug --session-profile one-shot",
            "cargo xtask differential compare --scenario rigid-world --preset oracle-release --session-profile one-shot",
            "cargo xtask differential replay --scenario rigid-world --preset oracle-debug --session-profile one-shot",
            "cargo xtask differential verify-determinism --scenario rigid-world --preset oracle-debug --runs 2",
            "cmake --build target/reference/oracle-asan-ubsan --target liquidfun-reference-protocol-tests",
            "ctest --test-dir target/reference/oracle-asan-ubsan --output-on-failure --no-tests=error -R '^liquidfun-reference-protocol$'",
            "cargo xtask differential compare --scenario rigid-world --preset oracle-asan-ubsan --session-profile one-shot",
            "retains failures for seven days",
            "`aggregate-mass-atomicity`",
            "`non-dynamic-contact-admission`",
            "`ignored-step-parameters`",
            "`rigid-action-bound-mismatch`",
            "`invalid-centered-inertia-boundary`",
            "`rigid-staging-not-integrated`",
            "`rigid-sanitizer-not-executed`",
        ],
    ),
    (
        "COMPATIBILITY.md",
        &[
            "| `subsystem.contacts-and-filtering` | `liquidfun/Box2D/Box2D/Dynamics/Contacts` | `liquidfun::dynamics::contacts` | applicable | yes | yes | yes | yes | yes | no | yes | no |",
            "| `subsystem.rigid-bodies-and-fixtures` | `liquidfun/Box2D/Box2D/Dynamics` | `liquidfun::dynamics` | applicable | yes | yes | yes | yes | yes | no | yes | no |",
            "| `subsystem.rigid-islands-and-solver` | `liquidfun/Box2D/Box2D/Dynamics` | `liquidfun::dynamics` | applicable | yes | yes | no | no | no | no | no | no |",
            "| `subsystem.world-operations-and-observation` | `liquidfun/Box2D/Box2D/Dynamics` | `liquidfun::world` | applicable | yes | yes | no | no | no | no | no | no |",
            "| `public-api.liquidfun-box2d-box2d-dynamics-b2body-h` | `liquidfun/Box2D/Box2D/Dynamics/b2Body.h` | `liquidfun::dynamics` | applicable | yes | yes | yes | yes | yes | no | yes | no |",
            "| `public-api.liquidfun-box2d-box2d-dynamics-b2fixture-h` | `liquidfun/Box2D/Box2D/Dynamics/b2Fixture.h` | `liquidfun::dynamics` | applicable | yes | yes | yes | yes | yes | no | yes | no |",
            "| `public-api.liquidfun-box2d-box2d-dynamics-b2contactmanager-h` | `liquidfun/Box2D/Box2D/Dynamics/b2ContactManager.h` | `liquidfun::dynamics` | applicable | yes | yes | yes | yes | yes | no | yes | no |",
            "| `public-api.liquidfun-box2d-box2d-dynamics-b2world-h` | `liquidfun/Box2D/Box2D/Dynamics/b2World.h` | `liquidfun::dynamics` | applicable | yes | yes | yes | yes | yes | no | yes | no |",
            "| `public-api.liquidfun-box2d-box2d-dynamics-contacts-b2circlecontact-h` | `liquidfun/Box2D/Box2D/Dynamics/Contacts/b2CircleContact.h` | `liquidfun::dynamics::contacts` | applicable | yes | yes | yes | yes | yes | no | yes | no |",
            "| `public-api.liquidfun-box2d-box2d-dynamics-contacts-b2contactsolver-h` | `liquidfun/Box2D/Box2D/Dynamics/Contacts/b2ContactSolver.h` | `liquidfun::dynamics::contacts` | applicable | yes | yes | yes | yes | yes | no | yes | no |",
            "| `public-api.liquidfun-box2d-box2d-dynamics-contacts-b2polygoncontact-h` | `liquidfun/Box2D/Box2D/Dynamics/Contacts/b2PolygonContact.h` | `liquidfun::dynamics::contacts` | applicable | yes | yes | no | no | no | no | no | no |",
            "| `source-area.liquidfun-box2d-box2d-dynamics` | `liquidfun/Box2D/Box2D/Dynamics` | `liquidfun::dynamics` | applicable | yes | yes | yes | yes | yes | no | yes | no |",
            "| `source-area.liquidfun-box2d-box2d-dynamics-contacts` | `liquidfun/Box2D/Box2D/Dynamics/Contacts` | `liquidfun::dynamics::contacts` | applicable | yes | yes | yes | yes | yes | no | yes | no |",
        ],
    ),
    (
        "README.md",
        &[
            "Phase 6 minimal rigid-world vertical slice",
            "phase6-v1",
            "non_colliding_body_fixture_lifecycle",
            "single_contact_lifecycle",
            "fixed 128-action step",
            "canonical D1 authority before every write",
            "ASan/UBSan lane executes",
            "Phase 7",
            "remain pending",
        ],
    ),
    (
        "protocol/fixtures/accepted/rigid-world-request.jsonl",
        &[
            "static_kinematic_overlap_rejected",
            "kinematic_kinematic_overlap_rejected",
        ],
    ),
];
const FORBIDDEN_PHASE5_CLAIMS: [&str; 9] = [
    "full parity",
    "production ready",
    "all platforms validated",
    "query order is guaranteed",
    "global epsilon",
    "cargo xtask differential d0",
    "packed contact keys are public",
    "dynamictree exposes public iteration",
    "phase 6 is complete",
];
const FORBIDDEN_PHASE6_CLAIMS: [&str; 15] = [
    "full rigid parity",
    "public durable contacts",
    "mutable shapes",
    "global epsilon",
    "general solver is implemented",
    "complete island solver",
    "forces are implemented",
    "sleeping is implemented",
    "ccd is implemented",
    "world queries are implemented",
    "world configuration is implemented",
    "joint solving is implemented",
    "platform validated",
    "raw contact identity",
    "raw proxy identity",
];

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
    check_document_contracts(&repository_root)?;
    println!(
        "docs verified: {} testing layers, {} Phase 4, {} Phase 5, and {} Phase 6 document contracts",
        LAYER_RULES.len(),
        DOCUMENT_CONTRACTS.len(),
        PHASE5_DOCUMENT_CONTRACTS.len(),
        PHASE6_DOCUMENT_CONTRACTS.len()
    );
    Ok(())
}

fn check_document_contracts(repository_root: &std::path::Path) -> Result<(), DocsError> {
    check_required_markers(repository_root, DOCUMENT_CONTRACTS, "phase4-contract")?;
    check_required_markers(
        repository_root,
        PHASE5_DOCUMENT_CONTRACTS,
        "phase5-contract",
    )?;
    check_required_markers(
        repository_root,
        PHASE6_DOCUMENT_CONTRACTS,
        "phase6-contract",
    )?;

    for relative_path in [
        "ARCHITECTURE.md",
        "TESTING.md",
        "COMPATIBILITY.md",
        "README.md",
    ] {
        let path = repository_root.join(relative_path);
        let contents = fs::read_to_string(&path).map_err(|error| {
            DocsError::new(
                "filesystem",
                format!("failed to read {}: {error}", path.display()),
            )
        })?;
        if contents.contains("/Users/") || contents.contains("C:\\Users\\") {
            return Err(DocsError::new(
                "local-path",
                format!("{relative_path} contains an absolute user path"),
            ));
        }
        let lowercase = contents.to_ascii_lowercase();
        if let Some(claim) = FORBIDDEN_PHASE5_CLAIMS
            .iter()
            .find(|claim| lowercase.contains(**claim))
        {
            return Err(DocsError::new(
                "phase5-overclaim",
                format!("{relative_path} contains forbidden claim `{claim}`"),
            ));
        }
        if let Some(claim) = FORBIDDEN_PHASE6_CLAIMS
            .iter()
            .find(|claim| lowercase.contains(**claim))
        {
            return Err(DocsError::new(
                "phase6-overclaim",
                format!("{relative_path} contains forbidden claim `{claim}`"),
            ));
        }
    }
    Ok(())
}

fn check_required_markers<const N: usize>(
    repository_root: &std::path::Path,
    contracts: [(&str, &[&str]); N],
    category: &'static str,
) -> Result<(), DocsError> {
    for (relative_path, required_markers) in contracts {
        let path = repository_root.join(relative_path);
        let contents = fs::read_to_string(&path).map_err(|error| {
            DocsError::new(
                "filesystem",
                format!("failed to read {}: {error}", path.display()),
            )
        })?;
        for marker in required_markers {
            if !contents.contains(marker) {
                return Err(DocsError::new(
                    category,
                    format!("{relative_path} must contain `{marker}`"),
                ));
            }
        }
    }
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
