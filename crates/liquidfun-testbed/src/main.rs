//! Private `LiquidFun` visual testbed and renderer capability command.

use std::env;
use std::io::{self, Write};
use std::path::PathBuf;

use liquidfun_testbed::{
    CapabilityOptions, run_capability_check,
    screenshot::{VisualContractOptions, run_visual_contract_check},
};

fn main() {
    if let Err(category) = run() {
        let _ = writeln!(io::stderr(), "liquidfun-testbed: {category}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), &'static str> {
    let command = parse_options(env::args().skip(1))?;
    match command {
        Command::Capability(options) => {
            let report = run_capability_check(&options)
                .map_err(liquidfun_testbed::CapabilityError::category)?;
            serde_json::to_writer(io::stdout().lock(), &report).map_err(|_| "report_encoding")?;
        }
        Command::VisualContract(options) => {
            let report = run_visual_contract_check(&options)
                .map_err(liquidfun_testbed::screenshot::VisualContractError::category)?;
            serde_json::to_writer(io::stdout().lock(), &report).map_err(|_| "report_encoding")?;
        }
    }
    writeln!(io::stdout()).map_err(|_| "output")?;
    Ok(())
}

enum Command {
    Capability(CapabilityOptions),
    VisualContract(VisualContractOptions),
}

fn parse_options(arguments: impl IntoIterator<Item = String>) -> Result<Command, &'static str> {
    let mut arguments = arguments.into_iter();
    let command = arguments.next().ok_or("missing command")?;
    if arguments.next().as_deref() != Some("--fixture") {
        return Err("expected --fixture");
    }
    let fixture = arguments.next().ok_or("missing fixture")?;
    if arguments.next().as_deref() != Some("--output") {
        return Err("expected --output");
    }
    let output = arguments.next().ok_or("missing output")?;
    if arguments.next().is_some() {
        return Err("unexpected argument");
    }
    match command.as_str() {
        "--capability-check" => Ok(Command::Capability(CapabilityOptions::new(
            PathBuf::from(fixture),
            PathBuf::from(output),
        ))),
        "--visual-contract-check" => Ok(Command::VisualContract(
            VisualContractOptions::new(
                PathBuf::from(fixture),
                PathBuf::from(output),
                option_env!("LIQUIDFUN_GIT_COMMIT").unwrap_or("Unavailable"),
            )
            .map_err(|_| "invalid provenance")?,
        )),
        _ => Err("expected --capability-check or --visual-contract-check"),
    }
}
