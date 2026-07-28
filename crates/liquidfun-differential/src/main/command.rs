use std::path::PathBuf;

use liquidfun_differential::{OraclePreset, SessionProfile};

use super::CliError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Action {
    Compare,
    Replay,
    Minimize,
}

pub(super) enum Input {
    Named(String),
    ExactRequest(PathBuf),
}

pub(super) struct CommandConfig {
    pub(super) action: Action,
    pub(super) input: Input,
    pub(super) preset: OraclePreset,
    pub(super) profile: SessionProfile,
}

impl CommandConfig {
    pub(super) fn parse(arguments: impl Iterator<Item = String>) -> Result<Self, CliError> {
        let mut arguments = arguments;
        let action = match arguments.next().as_deref() {
            Some("compare") => Action::Compare,
            Some("replay") => Action::Replay,
            Some("minimize") => Action::Minimize,
            _ => return Err(CliError::Usage(usage())),
        };
        let mut maybe_scenario = None;
        let mut maybe_exact_request = None;
        let mut maybe_preset = None;
        let mut maybe_profile = None;
        while let Some(option) = arguments.next() {
            let value = arguments.next().ok_or_else(|| CliError::Usage(usage()))?;
            match option.as_str() {
                "--scenario" if maybe_scenario.is_none() => maybe_scenario = Some(value),
                "--exact-request" if maybe_exact_request.is_none() => {
                    maybe_exact_request = Some(PathBuf::from(value));
                }
                "--preset" if maybe_preset.is_none() => maybe_preset = Some(parse_preset(&value)?),
                "--session-profile" if maybe_profile.is_none() => {
                    maybe_profile = Some(parse_profile(&value)?);
                }
                _ => return Err(CliError::Usage(usage())),
            }
        }
        let preset = maybe_preset.ok_or_else(|| CliError::Usage(usage()))?;
        let profile = maybe_profile.ok_or_else(|| CliError::Usage(usage()))?;
        let input = match (maybe_scenario, maybe_exact_request) {
            (Some(name), None) => Input::Named(name),
            (None, Some(path)) if action == Action::Replay => Input::ExactRequest(path),
            _ => return Err(CliError::Usage(usage())),
        };
        Ok(Self {
            action,
            input,
            preset,
            profile,
        })
    }
}

pub(super) fn parse_preset(value: &str) -> Result<OraclePreset, CliError> {
    match value {
        "oracle-debug" => Ok(OraclePreset::Debug),
        "oracle-release" => Ok(OraclePreset::Release),
        "oracle-asan-ubsan" => Ok(OraclePreset::AsanUbsan),
        _ => Err(CliError::Usage(usage())),
    }
}

pub(super) fn parse_profile(value: &str) -> Result<SessionProfile, CliError> {
    match value {
        "one-shot" => Ok(SessionProfile::OneShot),
        "reuse" => Ok(SessionProfile::Reuse),
        "sanitizer" => Ok(SessionProfile::Sanitizer),
        _ => Err(CliError::Usage(usage())),
    }
}

pub(super) fn usage() -> String {
    "usage: liquidfun-differential native-rigid-world --request <file>; or \
     <compare|replay|minimize> --scenario empty-world \
     --preset <oracle-debug|oracle-release|oracle-asan-ubsan> \
     --session-profile <one-shot|reuse|sanitizer>; replay also accepts --exact-request <file>"
        .to_owned()
}
