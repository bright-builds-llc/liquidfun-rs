//! Exploratory playground Dam Break native-versus-oracle step timing.

mod counts;
mod error;
mod identity;
mod pair;

pub(crate) use error::PlaygroundError;

const USAGE: &str = "Usage: cargo xtask playground dam-break-bench [--warmup <n>] [--steps <n>]";

/// Runs the exploratory playground Dam Break native-versus-oracle pair.
///
/// # Errors
///
/// Returns a closed error when arguments, the oracle build, either bench, or
/// host identity collection fails.
pub(crate) fn run(args: &[String]) -> Result<(), PlaygroundError> {
    let (command, command_args) = args
        .split_first()
        .ok_or_else(|| PlaygroundError::usage("expected `dam-break-bench`"))?;
    if command != "dam-break-bench" {
        return Err(PlaygroundError::usage(format!(
            "unknown playground command `{command}`"
        )));
    }
    pair::run(command_args)
}

#[cfg(test)]
mod tests {
    use super::{PlaygroundError, run};

    #[test]
    fn missing_subcommand_is_a_usage_error() {
        // Arrange / Act
        let result = run(&[]);

        // Assert
        assert_eq!(
            result,
            Err(PlaygroundError::usage("expected `dam-break-bench`"))
        );
    }

    #[test]
    fn unknown_subcommand_is_a_usage_error() {
        // Arrange / Act
        let result = run(&[String::from("nope")]);

        // Assert
        assert_eq!(
            result,
            Err(PlaygroundError::usage("unknown playground command `nope`"))
        );
    }
}
