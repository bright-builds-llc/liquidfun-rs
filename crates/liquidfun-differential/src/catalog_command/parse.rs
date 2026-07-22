//! Closed option, setting, and controller-script parsing.

use std::collections::BTreeMap;

use liquidfun_differential::{OraclePreset, SessionProfile};
use liquidfun_test_protocol::{
    CATALOG_MAXIMUM_ACTIONS, CATALOG_MAXIMUM_ITERATIONS, CatalogSlug, CheckpointId, FloatBits,
    RunSettings, ScenarioActionId,
};

const MAXIMUM_SCRIPT_BYTES: usize = 8 * 1024;
const EXIT_USAGE: u8 = 64;
const EXIT_SCENARIO: u8 = 65;
const EXIT_SETTINGS: u8 = 66;
const EXIT_SCRIPT: u8 = 67;
const EXIT_HARNESS_FAILURE: u8 = 3;
const EXIT_ORACLE_UNAVAILABLE: u8 = 69;

#[derive(Clone, Copy)]
pub(super) enum OutputMode {
    Human,
    Json,
}

impl OutputMode {
    pub(super) fn parse(value: &str) -> Result<Self, CatalogCliError> {
        match value {
            "human" => Ok(Self::Human),
            "json" => Ok(Self::Json),
            _ => Err(CatalogCliError::usage("output must be `human` or `json`")),
        }
    }
}

pub(super) struct ExecutionConfig {
    pub(super) scenario: CatalogSlug,
    pub(super) maybe_seed: Option<u64>,
    pub(super) settings: RunSettings,
    pub(super) preset: OraclePreset,
    pub(super) profile: SessionProfile,
    pub(super) output: OutputMode,
    pub(super) commands: CommandScript,
}

impl ExecutionConfig {
    pub(super) fn parse(args: &[String]) -> Result<Self, CatalogCliError> {
        const REQUIRED: [&str; 10] = [
            "--scenario",
            "--seed",
            "--timestep",
            "--velocity-iterations",
            "--position-iterations",
            "--particle-iterations",
            "--oracle-preset",
            "--session-profile",
            "--output",
            "--commands",
        ];
        let options = parse_options(args)?;
        require_shape(&options, &REQUIRED, &[])?;
        let scenario = CatalogSlug::new(required(&options, "--scenario")?.to_owned())
            .map_err(|_| CatalogCliError::scenario("invalid scenario slug"))?;
        let seed = required(&options, "--seed")?;
        let maybe_seed = if seed == "none" {
            None
        } else {
            Some(
                seed.parse::<u64>()
                    .map_err(|_| CatalogCliError::usage("invalid seed"))?,
            )
        };
        let timestep = required(&options, "--timestep")?
            .parse::<f32>()
            .map_err(|_| CatalogCliError::settings("invalid timestep"))?;
        let settings = RunSettings::new(
            FloatBits::from_f32(timestep),
            iterations(&options, "--velocity-iterations")?,
            iterations(&options, "--position-iterations")?,
            iterations(&options, "--particle-iterations")?,
        )
        .map_err(|_| CatalogCliError::settings("settings are outside reviewed bounds"))?;
        Ok(Self {
            scenario,
            maybe_seed,
            settings,
            preset: preset(required(&options, "--oracle-preset")?)?,
            profile: profile(required(&options, "--session-profile")?)?,
            output: OutputMode::parse(required(&options, "--output")?)?,
            commands: CommandScript::parse(required(&options, "--commands")?)?,
        })
    }
}

pub(super) enum CommandScript {
    Auto,
    Explicit(Vec<ControllerCommand>),
}

impl CommandScript {
    fn parse(value: &str) -> Result<Self, CatalogCliError> {
        if value == "auto" {
            return Ok(Self::Auto);
        }
        if value.is_empty() || value.len() > MAXIMUM_SCRIPT_BYTES {
            return Err(CatalogCliError::script("invalid command script size"));
        }
        let commands = value
            .split(',')
            .map(ControllerCommand::parse)
            .collect::<Result<Vec<_>, _>>()?;
        if commands.len() > CATALOG_MAXIMUM_ACTIONS {
            return Err(CatalogCliError::script(
                "command script exceeds 128 commands",
            ));
        }
        Ok(Self::Explicit(commands))
    }
}

pub(super) enum ControllerCommand {
    Pause,
    Resume,
    Step,
    Restart,
    ScenarioAction(ScenarioActionId),
    Capture(CheckpointId),
}

impl ControllerCommand {
    fn parse(value: &str) -> Result<Self, CatalogCliError> {
        match value {
            "pause" => Ok(Self::Pause),
            "resume" => Ok(Self::Resume),
            "step" => Ok(Self::Step),
            "restart" => Ok(Self::Restart),
            _ => {
                if let Some(value) = value.strip_prefix("scenario-action:") {
                    return ScenarioActionId::new(value.to_owned())
                        .map(Self::ScenarioAction)
                        .map_err(|_| CatalogCliError::script("invalid scenario action identity"));
                }
                if let Some(value) = value.strip_prefix("capture:") {
                    return CheckpointId::new(value.to_owned())
                        .map(Self::Capture)
                        .map_err(|_| CatalogCliError::script("invalid checkpoint identity"));
                }
                Err(CatalogCliError::script("unknown controller command"))
            }
        }
    }
}

pub(super) fn parse_options(args: &[String]) -> Result<BTreeMap<String, String>, CatalogCliError> {
    let mut options = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        if !pair[0].starts_with("--") || options.insert(pair[0].clone(), pair[1].clone()).is_some()
        {
            return Err(CatalogCliError::usage(
                "options must be unique option/value pairs",
            ));
        }
    }
    if !args.chunks_exact(2).remainder().is_empty() {
        return Err(CatalogCliError::usage("every option requires one value"));
    }
    Ok(options)
}

pub(super) fn require_shape(
    options: &BTreeMap<String, String>,
    required_options: &[&str],
    optional_options: &[&str],
) -> Result<(), CatalogCliError> {
    if required_options
        .iter()
        .all(|name| options.contains_key(*name))
        && options.keys().all(|name| {
            required_options.contains(&name.as_str()) || optional_options.contains(&name.as_str())
        })
    {
        return Ok(());
    }
    Err(CatalogCliError::usage(
        "options do not match the registered catalog command shape",
    ))
}

pub(super) fn required<'a>(
    options: &'a BTreeMap<String, String>,
    name: &str,
) -> Result<&'a str, CatalogCliError> {
    options
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| CatalogCliError::usage(format!("missing `{name}`")))
}

fn iterations(options: &BTreeMap<String, String>, name: &str) -> Result<u32, CatalogCliError> {
    let value = required(options, name)?
        .parse::<u32>()
        .map_err(|_| CatalogCliError::settings(format!("invalid `{name}`")))?;
    if !(1..=CATALOG_MAXIMUM_ITERATIONS).contains(&value) {
        return Err(CatalogCliError::settings(format!(
            "`{name}` must be from 1 to 1024"
        )));
    }
    Ok(value)
}

fn preset(value: &str) -> Result<OraclePreset, CatalogCliError> {
    match value {
        "oracle-debug" => Ok(OraclePreset::Debug),
        "oracle-release" => Ok(OraclePreset::Release),
        "oracle-asan-ubsan" => Ok(OraclePreset::AsanUbsan),
        _ => Err(CatalogCliError::usage("unregistered oracle preset")),
    }
}

fn profile(value: &str) -> Result<SessionProfile, CatalogCliError> {
    match value {
        "one-shot" => Ok(SessionProfile::OneShot),
        "reuse" => Ok(SessionProfile::Reuse),
        "sanitizer" => Ok(SessionProfile::Sanitizer),
        _ => Err(CatalogCliError::usage("unregistered session profile")),
    }
}

#[derive(Clone, Copy)]
pub(super) enum CatalogCliErrorKind {
    Usage,
    Scenario,
    Settings,
    Script,
    OracleUnavailable,
    Harness,
}

impl CatalogCliErrorKind {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Usage => "usage",
            Self::Scenario => "scenario",
            Self::Settings => "settings",
            Self::Script => "script",
            Self::OracleUnavailable => "oracle-unavailable",
            Self::Harness => "harness-failure",
        }
    }

    pub(super) const fn exit_code(self) -> u8 {
        match self {
            Self::Usage => EXIT_USAGE,
            Self::Scenario => EXIT_SCENARIO,
            Self::Settings => EXIT_SETTINGS,
            Self::Script => EXIT_SCRIPT,
            Self::OracleUnavailable => EXIT_ORACLE_UNAVAILABLE,
            Self::Harness => EXIT_HARNESS_FAILURE,
        }
    }
}

pub(super) struct CatalogCliError {
    pub(super) kind: CatalogCliErrorKind,
    pub(super) message: String,
}

impl CatalogCliError {
    fn new(kind: CatalogCliErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub(super) fn usage(message: impl Into<String>) -> Self {
        Self::new(CatalogCliErrorKind::Usage, message)
    }

    pub(super) fn scenario(message: impl Into<String>) -> Self {
        Self::new(CatalogCliErrorKind::Scenario, message)
    }

    pub(super) fn settings(message: impl Into<String>) -> Self {
        Self::new(CatalogCliErrorKind::Settings, message)
    }

    pub(super) fn script(message: impl Into<String>) -> Self {
        Self::new(CatalogCliErrorKind::Script, message)
    }

    pub(super) fn oracle(message: impl Into<String>) -> Self {
        Self::new(CatalogCliErrorKind::OracleUnavailable, message)
    }

    pub(super) fn harness(message: impl Into<String>) -> Self {
        Self::new(CatalogCliErrorKind::Harness, message)
    }
}
