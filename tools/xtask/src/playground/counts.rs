use super::PlaygroundError;

pub(super) const DEFAULT_WARMUP_STEPS: u32 = 60;
pub(super) const DEFAULT_MEASURED_STEPS: u32 = 600;

pub(super) struct BenchCounts {
    pub(super) warmup_steps: u32,
    pub(super) measured_steps: u32,
}

pub(super) fn parse_counts(args: &[String]) -> Result<BenchCounts, PlaygroundError> {
    let mut warmup_steps = DEFAULT_WARMUP_STEPS;
    let mut measured_steps = DEFAULT_MEASURED_STEPS;
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
    Ok(BenchCounts {
        warmup_steps,
        measured_steps,
    })
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
    use super::parse_counts;

    #[test]
    fn parse_counts_defaults_to_locked_warmup_and_steps() {
        // Arrange / Act
        let counts = parse_counts(&[]).expect("defaults should parse");

        // Assert
        assert_eq!(counts.warmup_steps, 60);
        assert_eq!(counts.measured_steps, 600);
    }
}
