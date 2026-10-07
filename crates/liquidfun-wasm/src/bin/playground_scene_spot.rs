//! Native all-catalog playground scene survey timer.

use std::env;
use std::error::Error;
use std::process::ExitCode;

use liquidfun_wasm::{
    run_scene_spot,
    scene_spot::{DEFAULT_MEASURED_STEPS, DEFAULT_RUNS, DEFAULT_WARMUP_STEPS},
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
    let spot_args = parse_args(&args)?;
    let scene_refs: Vec<&str> = spot_args.scenes.iter().map(String::as_str).collect();
    let samples = run_scene_spot(
        spot_args.warmup_steps,
        spot_args.measured_steps,
        spot_args.runs,
        &scene_refs,
    )?;
    for sample in samples {
        println!("{}", sample.to_json());
    }
    Ok(())
}

/// Parsed survey flags; an empty `scenes` list means the full catalog.
struct SpotArgs {
    warmup_steps: u32,
    measured_steps: u32,
    runs: u32,
    scenes: Vec<String>,
}

fn parse_args(args: &[String]) -> Result<SpotArgs, Box<dyn Error>> {
    let mut warmup_steps = DEFAULT_WARMUP_STEPS;
    let mut measured_steps = DEFAULT_MEASURED_STEPS;
    let mut runs = DEFAULT_RUNS;
    let mut scenes = Vec::new();
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
            "--runs" => {
                runs = parse_flag_value(args, index, "--runs")?;
                index += 2;
            }
            "--scene" => {
                let Some(scene) = args.get(index + 1).filter(|value| !value.starts_with("--"))
                else {
                    return Err("--scene requires a scene id".into());
                };
                scenes.push(scene.clone());
                index += 2;
            }
            unknown => {
                return Err(format!(
                    "unknown argument `{unknown}`; expected `--warmup <n>`, `--steps <n>`, `--runs <n>`, and/or `--scene <id>`"
                )
                .into());
            }
        }
    }
    Ok(SpotArgs {
        warmup_steps,
        measured_steps,
        runs,
        scenes,
    })
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
