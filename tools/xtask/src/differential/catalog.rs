//! Closed argument parsing for the renderer-free scenario catalog runner.

use std::collections::BTreeMap;

use liquidfun_test_protocol::{
    CATALOG_MAXIMUM_ACTIONS, CATALOG_MAXIMUM_ITERATIONS, CatalogSlug, CheckpointId,
    ScenarioActionId, reviewed_scenario_catalog,
};

use super::{DifferentialError, repository_root, run_differential};

const PRESETS: [&str; 3] = ["oracle-debug", "oracle-release", "oracle-asan-ubsan"];
const PROFILES: [&str; 3] = ["one-shot", "reuse", "sanitizer"];
const OUTPUTS: [&str; 2] = ["human", "json"];
const MAXIMUM_SCRIPT_BYTES: usize = 8 * 1024;

pub(super) fn run(args: &[String]) -> Result<(), DifferentialError> {
    let arguments = parse(args)?;
    run_differential(&repository_root()?, &arguments)
}

fn parse(args: &[String]) -> Result<Vec<String>, DifferentialError> {
    let Some((action, tail)) = args.split_first() else {
        return Err(usage("missing catalog action"));
    };
    match action.as_str() {
        "list" => parse_list(tail),
        "inspect" => parse_inspect(tail),
        "run" | "replay" | "compare" => parse_execution(action, tail),
        unknown => Err(usage(format!("unknown catalog action `{unknown}`"))),
    }
}

fn parse_list(args: &[String]) -> Result<Vec<String>, DifferentialError> {
    if !args.is_empty() {
        return Err(usage("catalog list does not accept arguments"));
    }
    Ok(vec!["catalog".to_owned(), "list".to_owned()])
}

fn parse_inspect(args: &[String]) -> Result<Vec<String>, DifferentialError> {
    let options = parse_options(args)?;
    require_shape(&options, &["--scenario", "--output"], &[])?;
    let scenario = scenario(&options)?;
    let output = allowed(&options, "--output", &OUTPUTS)?;
    Ok(arguments(
        &["catalog", "inspect"],
        &[("--scenario", scenario), ("--output", output)],
    ))
}

fn parse_execution(action: &str, args: &[String]) -> Result<Vec<String>, DifferentialError> {
    const REQUIRED: [&str; 8] = [
        "--scenario",
        "--timestep",
        "--velocity-iterations",
        "--position-iterations",
        "--particle-iterations",
        "--oracle-preset",
        "--session-profile",
        "--output",
    ];
    const OPTIONAL: [&str; 2] = ["--seed", "--commands"];
    let options = parse_options(args)?;
    require_shape(&options, &REQUIRED, &OPTIONAL)?;
    let scenario = scenario(&options)?;
    let seed = options.get("--seed").map_or("none", String::as_str);
    validate_seed(seed)?;
    let timestep = required(&options, "--timestep")?;
    validate_timestep(timestep)?;
    let velocity = validated_iterations(&options, "--velocity-iterations")?;
    let position = validated_iterations(&options, "--position-iterations")?;
    let particle = validated_iterations(&options, "--particle-iterations")?;
    let preset = allowed(&options, "--oracle-preset", &PRESETS)?;
    let profile = allowed(&options, "--session-profile", &PROFILES)?;
    let output = allowed(&options, "--output", &OUTPUTS)?;
    let commands = options.get("--commands").map_or("auto", String::as_str);
    validate_script(commands)?;
    Ok(arguments(
        &["catalog", action],
        &[
            ("--scenario", scenario),
            ("--seed", seed),
            ("--timestep", timestep),
            ("--velocity-iterations", velocity),
            ("--position-iterations", position),
            ("--particle-iterations", particle),
            ("--oracle-preset", preset),
            ("--session-profile", profile),
            ("--output", output),
            ("--commands", commands),
        ],
    ))
}

fn parse_options(args: &[String]) -> Result<BTreeMap<String, String>, DifferentialError> {
    let mut options = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        if !pair[0].starts_with("--") || options.insert(pair[0].clone(), pair[1].clone()).is_some()
        {
            return Err(usage("catalog options must be unique option/value pairs"));
        }
    }
    if !args.chunks_exact(2).remainder().is_empty() {
        return Err(usage("every catalog option requires one value"));
    }
    Ok(options)
}

fn require_shape(
    options: &BTreeMap<String, String>,
    required_options: &[&str],
    optional_options: &[&str],
) -> Result<(), DifferentialError> {
    if required_options
        .iter()
        .all(|option| options.contains_key(*option))
        && options.keys().all(|option| {
            required_options.contains(&option.as_str())
                || optional_options.contains(&option.as_str())
        })
    {
        return Ok(());
    }
    Err(usage(
        "catalog command options do not match the registered command shape",
    ))
}

fn scenario(options: &BTreeMap<String, String>) -> Result<&str, DifferentialError> {
    let value = required(options, "--scenario")?;
    let slug = CatalogSlug::new(value.to_owned()).map_err(|_| {
        DifferentialError::new("catalog-scenario", "scenario must be a stable catalog slug")
    })?;
    let catalog = reviewed_scenario_catalog().map_err(|error| {
        DifferentialError::new("catalog-scenario", format!("catalog unavailable: {error}"))
    })?;
    if catalog
        .definitions()
        .iter()
        .any(|definition| definition.slug() == &slug)
    {
        return Ok(value);
    }
    Err(DifferentialError::new(
        "catalog-scenario",
        format!("unknown scenario slug `{value}`"),
    ))
}

fn validate_seed(value: &str) -> Result<(), DifferentialError> {
    if value == "none" || value.parse::<u64>().is_ok() {
        return Ok(());
    }
    Err(usage("seed must be `none` or an unsigned base-10 integer"))
}

fn validate_timestep(value: &str) -> Result<(), DifferentialError> {
    let valid = value
        .parse::<f32>()
        .is_ok_and(|timestep| timestep.is_finite() && timestep > 0.0);
    if valid {
        return Ok(());
    }
    Err(DifferentialError::new(
        "catalog-settings",
        "timestep must be finite and greater than zero",
    ))
}

fn validated_iterations<'a>(
    options: &'a BTreeMap<String, String>,
    name: &str,
) -> Result<&'a str, DifferentialError> {
    let value = required(options, name)?;
    if value
        .parse::<u32>()
        .is_ok_and(|count| (1..=CATALOG_MAXIMUM_ITERATIONS).contains(&count))
    {
        return Ok(value);
    }
    Err(DifferentialError::new(
        "catalog-settings",
        format!("`{name}` must be an integer from 1 to 1024"),
    ))
}

fn validate_script(script: &str) -> Result<(), DifferentialError> {
    if script == "auto" {
        return Ok(());
    }
    if script.is_empty() || script.len() > MAXIMUM_SCRIPT_BYTES {
        return Err(script_error());
    }
    let commands = script.split(',').collect::<Vec<_>>();
    if commands.len() > CATALOG_MAXIMUM_ACTIONS
        || commands.iter().any(|command| !valid_command(command))
    {
        return Err(script_error());
    }
    Ok(())
}

fn valid_command(command: &str) -> bool {
    if matches!(command, "pause" | "resume" | "step" | "restart") {
        return true;
    }
    if let Some(action_id) = command.strip_prefix("scenario-action:") {
        return ScenarioActionId::new(action_id.to_owned()).is_ok();
    }
    if let Some(checkpoint_id) = command.strip_prefix("capture:") {
        return CheckpointId::new(checkpoint_id.to_owned()).is_ok();
    }
    false
}

fn required<'a>(
    options: &'a BTreeMap<String, String>,
    name: &str,
) -> Result<&'a str, DifferentialError> {
    options
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| usage(format!("missing required catalog option `{name}`")))
}

fn allowed<'a>(
    options: &'a BTreeMap<String, String>,
    name: &str,
    allowed_values: &[&str],
) -> Result<&'a str, DifferentialError> {
    let value = required(options, name)?;
    if allowed_values.contains(&value) {
        return Ok(value);
    }
    Err(usage(format!("unregistered value `{value}` for `{name}`")))
}

fn arguments(prefix: &[&str], options: &[(&str, &str)]) -> Vec<String> {
    prefix
        .iter()
        .copied()
        .chain(options.iter().flat_map(|(name, value)| [*name, *value]))
        .map(str::to_owned)
        .collect()
}

fn usage(message: impl Into<String>) -> DifferentialError {
    DifferentialError::new("catalog-usage", message)
}

fn script_error() -> DifferentialError {
    DifferentialError::new(
        "catalog-script",
        "command script must contain at most 128 closed controller commands",
    )
}
