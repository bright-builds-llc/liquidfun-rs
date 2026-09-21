//! Exploratory playground Dam Break native-versus-oracle step timing.

mod counts;
mod error;
mod identity;
mod pair;
mod profile;
mod stamp;
mod timers;

pub(crate) use error::PlaygroundError;

const USAGE: &str = "Usage: cargo xtask playground <dam-break-bench|dam-break-profile|dam-break-timers> [--warmup <n>] [--steps <n>]";

/// Runs exploratory playground Dam Break pair, CPU-profile, or timer commands.
///
/// # Errors
///
/// Returns a closed error when arguments, the oracle build, either bench,
/// samply, parent timers, or host identity collection fails.
pub(crate) fn run(args: &[String]) -> Result<(), PlaygroundError> {
    let (command, command_args) = args.split_first().ok_or_else(|| {
        PlaygroundError::usage(
            "expected `dam-break-bench`, `dam-break-profile`, or `dam-break-timers`",
        )
    })?;
    match command.as_str() {
        "dam-break-bench" => pair::run(command_args),
        "dam-break-profile" => profile::run(command_args),
        "dam-break-timers" => timers::run(command_args),
        _ => Err(PlaygroundError::usage(format!(
            "unknown playground command `{command}`"
        ))),
    }
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
            Err(PlaygroundError::usage(
                "expected `dam-break-bench`, `dam-break-profile`, or `dam-break-timers`"
            ))
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
