//! Private `LiquidFun` visual testbed and renderer capability command.

use std::env;
use std::io::{self, Write};
use std::path::PathBuf;

use liquidfun_testbed::{CapabilityOptions, run_capability_check};

fn main() {
    if let Err(category) = run() {
        let _ = writeln!(io::stderr(), "liquidfun-testbed: {category}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), &'static str> {
    let options = parse_options(env::args().skip(1))?;
    let report =
        run_capability_check(&options).map_err(liquidfun_testbed::CapabilityError::category)?;
    serde_json::to_writer(io::stdout().lock(), &report).map_err(|_| "report_encoding")?;
    writeln!(io::stdout()).map_err(|_| "output")?;
    Ok(())
}

fn parse_options(
    arguments: impl IntoIterator<Item = String>,
) -> Result<CapabilityOptions, &'static str> {
    let mut arguments = arguments.into_iter();
    if arguments.next().as_deref() != Some("--capability-check") {
        return Err("expected --capability-check");
    }
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
    Ok(CapabilityOptions::new(
        PathBuf::from(fixture),
        PathBuf::from(output),
    ))
}
