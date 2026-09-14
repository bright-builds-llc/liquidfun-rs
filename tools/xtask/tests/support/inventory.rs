//! Complete checked inventory inputs for isolated report and attestation fixtures.

use std::{fs, io, path::Path};

const INPUTS: [&str; 18] = [
    "reference/upstream-lock.toml",
    "reference/compatibility.json",
    "reference/discovery.json",
    "reference/upstream-corpus.json",
    "reference/artifacts/manifest.toml",
    "reference/platform/support.json",
    "reference/performance/manifest.toml",
    "reference/coverage/contract.json",
    "reference/regressions/manifest.toml",
    "reference/artifacts/phase11/exact-ref.json",
    "reference/scenario-catalog.json",
    "reference/artifacts/phase11/scenario-mappings.json",
    "crates/liquidfun-differential/tests/fixtures/catalog/phase11-v1.json",
    "crates/liquidfun-differential/tests/fixtures/catalog/cases/particle-groups.jsonl",
    "crates/liquidfun-differential/tests/fixtures/catalog/cases/queries-callbacks-mutations.jsonl",
    "crates/liquidfun-differential/tests/fixtures/catalog/cases/rigid-joint-rope.jsonl",
    "crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/phase9-v1.json",
    "crates/liquidfun-differential/tests/fixtures/rigid_world/phase10/phase10-v1.json",
];

pub(super) fn copy_inputs(source: &Path, destination: &Path) -> io::Result<()> {
    for relative in INPUTS {
        copy(source, destination, Path::new(relative))?;
    }
    for entry in fs::read_dir(source.join("protocol/tolerances"))? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            copy(
                source,
                destination,
                &Path::new("protocol/tolerances").join(entry.file_name()),
            )?;
        }
    }
    let report = non_ready_report(&fs::read_to_string(source.join("COMPATIBILITY.md"))?)?;
    fs::write(destination.join("COMPATIBILITY.md"), report)
}

fn copy(source: &Path, destination: &Path, relative: &Path) -> io::Result<()> {
    let path = destination.join(relative);
    fs::create_dir_all(
        path.parent()
            .ok_or_else(|| io::Error::other("fixture input needs a parent"))?,
    )?;
    fs::copy(source.join(relative), path)?;
    Ok(())
}

fn non_ready_report(contents: &str) -> io::Result<String> {
    let heading = "## Release readiness closure\n\n";
    let start = contents
        .find(heading)
        .ok_or_else(|| io::Error::other("missing readiness heading"))?
        + heading.len();
    let end = contents[start..]
        .find("| Closure measure |")
        .ok_or_else(|| io::Error::other("missing closure table"))?
        + start;
    // This is explicit test input for the stable default public-copy contract.
    let default = "Status: **not release-ready**. The compatibility ledger has zero unexplained gaps, but no completed full-SHA `release-candidate` workflow run and accepted frozen-source attestation exist. This closure is derived from exact identity joins; local command success never promotes evidence or substitutes for run-bound release attestation.\n\n";
    Ok(format!(
        "{}{default}{}",
        &contents[..start],
        &contents[end..]
    ))
}
