//! Private repository orchestration for `liquidfun-rs`.

mod inventory;
mod package;
mod provenance;
mod upstream;

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::process::ExitCode;

const USAGE: &str = r"Usage: cargo xtask <command> [arguments]

Commands:
  upstream    Manage the pinned upstream oracle
  inventory   Manage the compatibility inventory
  provenance  Validate provenance records
  package     Validate the publishable package
  check       Run the aggregate repository checks";

#[derive(Debug, PartialEq, Eq)]
enum XtaskError {
    Usage { message: String },
    NotImplemented { command: &'static str },
    Upstream(upstream::UpstreamError),
}

impl XtaskError {
    fn usage(message: impl Into<String>) -> Self {
        Self::Usage {
            message: message.into(),
        }
    }

    const fn not_implemented(command: &'static str) -> Self {
        Self::NotImplemented { command }
    }
}

impl Display for XtaskError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage { message } => write!(formatter, "{message}\n\n{USAGE}"),
            Self::NotImplemented { command } => {
                write!(
                    formatter,
                    "command `{command}` is not implemented by this plan"
                )
            }
            Self::Upstream(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for XtaskError {}

fn dispatch(args: &[String]) -> Result<(), XtaskError> {
    let Some((command, command_args)) = args.split_first() else {
        return Err(XtaskError::usage("missing command"));
    };

    match command.as_str() {
        "--help" | "-h" => {
            println!("{USAGE}");
            Ok(())
        }
        "upstream" => upstream::run(command_args).map_err(XtaskError::Upstream),
        "inventory" => inventory::run(command_args),
        "provenance" => provenance::run(command_args),
        "package" => package::run(command_args),
        "check" => Err(XtaskError::not_implemented("check")),
        unknown => Err(XtaskError::usage(format!("unknown command `{unknown}`"))),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match dispatch(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{XtaskError, dispatch};

    #[test]
    fn missing_command_returns_usage_error() {
        // Arrange
        let args = Vec::new();

        // Act
        let result = dispatch(&args);

        // Assert
        assert_eq!(result, Err(XtaskError::usage("missing command")));
    }

    #[test]
    fn unknown_command_returns_usage_error() {
        // Arrange
        let args = vec!["unknown".to_owned()];

        // Act
        let result = dispatch(&args);

        // Assert
        assert_eq!(result, Err(XtaskError::usage("unknown command `unknown`")));
    }

    #[test]
    fn help_returns_success() {
        // Arrange
        let args = vec!["--help".to_owned()];

        // Act
        let result = dispatch(&args);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn module_commands_delegate_to_matching_module() {
        // Arrange
        let commands = ["inventory", "provenance", "package"];

        for command in commands {
            // Act
            let result = dispatch(&[command.to_owned()]);

            // Assert
            assert_eq!(result, Err(XtaskError::not_implemented(command)));
        }
    }

    #[test]
    fn check_command_returns_typed_placeholder_error() {
        // Arrange
        let args = vec!["check".to_owned()];

        // Act
        let result = dispatch(&args);

        // Assert
        assert_eq!(result, Err(XtaskError::not_implemented("check")));
    }
}
