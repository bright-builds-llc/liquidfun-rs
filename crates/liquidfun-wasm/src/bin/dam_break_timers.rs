//! Native Dam Break Medium/Normal parent-phase timer.

use std::env;
use std::error::Error;
use std::process::ExitCode;

use liquidfun_wasm::{
    dam_break_bench::{DEFAULT_MEASURED_STEPS, DEFAULT_WARMUP_STEPS},
    run_dam_break_timers,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let (warmup_steps, measured_steps) = parse_counts(&args)?;
    let report = run_dam_break_timers(warmup_steps, measured_steps)?;
    println!("{}", report.to_json());
    Ok(())
}

fn parse_counts(args: &[String]) -> Result<(u32, u32), Box<dyn Error>> {
    let mut warmup_steps = DEFAULT_WARMUP_STEPS;
    let mut measured_steps = DEFAULT_MEASURED_STEPS;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--warmup" => {
                warmup_steps = parse_flag_value(args, index, "--warmup")?;
                index += 2;
            }
            "--steps" => {
                measured_steps = parse_flag_value(args, index, "--steps")?;
                index += 2;
            }
            unknown => {
                return Err(format!(
                    "unknown argument `{unknown}`; expected `--warmup <n>` and/or `--steps <n>`"
                )
                .into());
            }
        }
    }
    Ok((warmup_steps, measured_steps))
}

fn parse_flag_value(args: &[String], index: usize, flag: &str) -> Result<u32, Box<dyn Error>> {
    let Some(raw) = args.get(index + 1) else {
        return Err(format!("{flag} requires a non-negative integer").into());
    };
    let value = raw
        .parse::<u32>()
        .map_err(|_error| format!("{flag} requires a non-negative integer"))?;
    Ok(value)
}
