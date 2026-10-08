//! `scene-spot` flag parsing: counts plus repeatable catalog-checked `--scene` ids.

use crate::playground::PlaygroundError;

const DEFAULT_WARMUP_STEPS: u32 = 60;
const DEFAULT_MEASURED_STEPS: u32 = 120;
const DEFAULT_RUNS: u32 = 3;

/// Parsed `scene-spot` flags; an empty `scenes` list means the full catalog.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct SpotArgs {
    pub(super) warmup_steps: u32,
    pub(super) measured_steps: u32,
    pub(super) runs: u32,
    pub(super) scenes: Vec<String>,
}

/// Parses `--warmup`, `--steps`, `--runs` and repeatable `--scene <id>` against `catalog`.
pub(super) fn parse_spot_args(
    args: &[String],
    catalog: &[&str],
) -> Result<SpotArgs, PlaygroundError> {
    let mut warmup_steps = DEFAULT_WARMUP_STEPS;
    let mut measured_steps = DEFAULT_MEASURED_STEPS;
    let mut runs = DEFAULT_RUNS;
    let mut scenes: Vec<String> = Vec::new();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--warmup" => {
                warmup_steps = parse_u32_flag(args, index, "--warmup")?;
                index += 2;
            }
            "--steps" => {
                measured_steps = parse_u32_flag(args, index, "--steps")?;
                index += 2;
            }
            "--runs" => {
                runs = parse_u32_flag(args, index, "--runs")?;
                index += 2;
            }
            "--scene" => {
                let scene = parse_scene_flag(args, index, catalog, &scenes)?;
                scenes.push(scene);
                index += 2;
            }
            unknown => {
                return Err(PlaygroundError::usage(format!(
                    "unknown argument `{unknown}`"
                )));
            }
        }
    }
    if measured_steps == 0 {
        return Err(PlaygroundError::usage("--steps must be greater than 0"));
    }
    if runs == 0 {
        return Err(PlaygroundError::usage("--runs must be greater than 0"));
    }
    Ok(SpotArgs {
        warmup_steps,
        measured_steps,
        runs,
        scenes,
    })
}

fn parse_scene_flag(
    args: &[String],
    index: usize,
    catalog: &[&str],
    seen: &[String],
) -> Result<String, PlaygroundError> {
    let Some(id) = args.get(index + 1).filter(|value| !value.starts_with("--")) else {
        return Err(PlaygroundError::usage("--scene requires a scene id"));
    };
    if !catalog.contains(&id.as_str()) {
        return Err(PlaygroundError::usage(format!("unknown --scene `{id}`")));
    }
    if seen.contains(id) {
        return Err(PlaygroundError::usage(format!("duplicate --scene `{id}`")));
    }
    Ok(id.clone())
}

fn parse_u32_flag(args: &[String], index: usize, flag: &str) -> Result<u32, PlaygroundError> {
    let Some(raw) = args.get(index + 1) else {
        return Err(PlaygroundError::usage(format!(
            "{flag} requires a non-negative integer"
        )));
    };
    raw.parse::<u32>()
        .map_err(|_error| PlaygroundError::usage(format!("{flag} requires a non-negative integer")))
}

#[cfg(test)]
mod tests {
    use super::{SpotArgs, parse_spot_args};
    use crate::playground::PlaygroundError;

    const CATALOG: [&str; 3] = ["soup", "fountain", "dam-break"];

    fn owned(args: &[&str]) -> Vec<String> {
        args.iter().map(|arg| (*arg).to_owned()).collect()
    }

    #[test]
    fn parse_spot_args_collects_repeated_scenes() {
        // Arrange
        let args = owned(&["--scene", "soup", "--scene", "fountain"]);

        // Act
        let result = parse_spot_args(&args, &CATALOG);

        // Assert
        assert_eq!(
            result,
            Ok(SpotArgs {
                warmup_steps: 60,
                measured_steps: 120,
                runs: 3,
                scenes: owned(&["soup", "fountain"]),
            })
        );
    }

    #[test]
    fn parse_spot_args_rejects_unknown_scene() {
        // Arrange
        let args = owned(&["--scene", "nope"]);

        // Act
        let result = parse_spot_args(&args, &CATALOG);

        // Assert
        let error = result.expect_err("unknown scene should fail");
        assert!(error.to_string().contains("nope"), "{error}");
        assert!(
            error.to_string().starts_with("playground/usage:"),
            "{error}"
        );
    }

    #[test]
    fn parse_spot_args_rejects_duplicate_scene() {
        // Arrange
        let args = owned(&["--scene", "soup", "--scene", "soup"]);

        // Act
        let result = parse_spot_args(&args, &CATALOG);

        // Assert
        assert_eq!(
            result,
            Err(PlaygroundError::usage("duplicate --scene `soup`"))
        );
    }

    #[test]
    fn parse_spot_args_rejects_missing_scene_value() {
        // Arrange
        let args = owned(&["--runs", "1", "--scene"]);

        // Act
        let result = parse_spot_args(&args, &CATALOG);

        // Assert
        assert_eq!(
            result,
            Err(PlaygroundError::usage("--scene requires a scene id"))
        );
    }

    #[test]
    fn parse_spot_args_rejects_zero_runs() {
        // Arrange
        let args = owned(&["--runs", "0"]);

        // Act
        let result = parse_spot_args(&args, &CATALOG).map(|parsed| parsed.runs);

        // Assert
        assert_eq!(
            result,
            Err(PlaygroundError::usage("--runs must be greater than 0"))
        );
    }

    #[test]
    fn parse_spot_args_defaults_to_three_runs() {
        // Arrange
        let args: [String; 0] = [];

        // Act
        let result = parse_spot_args(&args, &CATALOG).map(|parsed| parsed.runs);

        // Assert
        assert_eq!(result, Ok(3));
    }
}
