//! `scene-fingerprint-gate` flag parsing: `--baseline`, and at most one of `--spot` or `--stamp`.

use std::path::PathBuf;

use crate::playground::PlaygroundError;
use crate::playground::stamp;

/// Where the observed fingerprints come from.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum ObservedSource {
    /// Run a fresh full-catalog `scene-spot` capture.
    Fresh,
    /// Read an existing `scene-spot.json` path.
    Spot(PathBuf),
    /// Read `target/dam-break-perf/<stamp>/scene-spot.json`.
    Stamp(String),
}

/// Parsed gate flags; `maybe_baseline` of `None` means the committed baseline.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct GateArgs {
    pub(super) maybe_baseline: Option<PathBuf>,
    pub(super) source: ObservedSource,
}

/// Parses gate flags. `--stamp` must be a filename-safe UTC stamp.
pub(super) fn parse_gate_args(args: &[String]) -> Result<GateArgs, PlaygroundError> {
    let mut maybe_baseline: Option<String> = None;
    let mut maybe_spot: Option<String> = None;
    let mut maybe_stamp: Option<String> = None;
    let mut index = 0;
    while index < args.len() {
        let flag = args[index].as_str();
        let slot = match flag {
            "--baseline" => &mut maybe_baseline,
            "--spot" => &mut maybe_spot,
            "--stamp" => &mut maybe_stamp,
            unknown => {
                return Err(PlaygroundError::usage(format!(
                    "unknown argument `{unknown}`"
                )));
            }
        };
        let Some(value) = args.get(index + 1).filter(|value| !value.starts_with("--")) else {
            return Err(PlaygroundError::usage(format!("{flag} requires a value")));
        };
        if slot.is_some() {
            return Err(PlaygroundError::usage(format!("{flag} may be given once")));
        }
        *slot = Some(value.clone());
        index += 2;
    }
    let source = match (maybe_spot, maybe_stamp) {
        (Some(_), Some(_)) => {
            return Err(PlaygroundError::usage(
                "--spot and --stamp cannot be used together",
            ));
        }
        (Some(spot), None) => ObservedSource::Spot(PathBuf::from(spot)),
        (None, Some(stamp_name)) => ObservedSource::Stamp(stamp::parse_stamp_name(&stamp_name)?),
        (None, None) => ObservedSource::Fresh,
    };
    Ok(GateArgs {
        maybe_baseline: maybe_baseline.map(PathBuf::from),
        source,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{GateArgs, ObservedSource, parse_gate_args};
    use crate::playground::PlaygroundError;

    fn strings(args: &[&str]) -> Vec<String> {
        args.iter().map(|arg| (*arg).to_owned()).collect()
    }

    #[test]
    fn no_flags_mean_a_fresh_capture_against_the_default_baseline() {
        // Arrange / Act
        let result = parse_gate_args(&[]);

        // Assert
        assert_eq!(
            result,
            Ok(GateArgs {
                maybe_baseline: None,
                source: ObservedSource::Fresh,
            })
        );
    }

    #[test]
    fn baseline_and_spot_paths_are_kept() {
        // Arrange
        let args = strings(&["--baseline", "b.json", "--spot", "s.json"]);

        // Act
        let result = parse_gate_args(&args);

        // Assert
        assert_eq!(
            result,
            Ok(GateArgs {
                maybe_baseline: Some(PathBuf::from("b.json")),
                source: ObservedSource::Spot(PathBuf::from("s.json")),
            })
        );
    }

    #[test]
    fn valid_stamp_is_accepted() {
        // Arrange
        let args = strings(&["--stamp", "2026-10-14T15-53-11Z"]);

        // Act
        let result = parse_gate_args(&args).map(|parsed| parsed.source);

        // Assert
        assert_eq!(
            result,
            Ok(ObservedSource::Stamp(String::from("2026-10-14T15-53-11Z")))
        );
    }

    #[test]
    fn traversal_stamp_is_rejected() {
        // Arrange
        let args = strings(&["--stamp", "../../etc"]);

        // Act
        let result = parse_gate_args(&args);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn spot_and_stamp_together_are_a_usage_error() {
        // Arrange
        let args = strings(&["--spot", "s.json", "--stamp", "2026-10-14T15-53-11Z"]);

        // Act
        let result = parse_gate_args(&args);

        // Assert
        assert_eq!(
            result,
            Err(PlaygroundError::usage(
                "--spot and --stamp cannot be used together"
            ))
        );
    }

    #[test]
    fn unknown_flag_is_a_usage_error() {
        // Arrange
        let args = strings(&["--scene", "dam-break"]);

        // Act
        let result = parse_gate_args(&args);

        // Assert
        assert_eq!(
            result,
            Err(PlaygroundError::usage("unknown argument `--scene`"))
        );
    }

    #[test]
    fn missing_value_is_a_usage_error() {
        // Arrange
        let args = strings(&["--spot"]);

        // Act
        let result = parse_gate_args(&args);

        // Assert
        assert_eq!(
            result,
            Err(PlaygroundError::usage("--spot requires a value"))
        );
    }

    #[test]
    fn repeated_flag_is_a_usage_error() {
        // Arrange
        let args = strings(&["--baseline", "a.json", "--baseline", "b.json"]);

        // Act
        let result = parse_gate_args(&args);

        // Assert
        assert_eq!(
            result,
            Err(PlaygroundError::usage("--baseline may be given once"))
        );
    }
}
