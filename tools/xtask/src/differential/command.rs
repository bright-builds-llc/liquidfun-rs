#[allow(
    clippy::wildcard_imports,
    reason = "this split module shares its parent private contract"
)]
use super::*;

pub(super) fn parse_invocation(args: &[String]) -> Result<RunnerInvocation, DifferentialError> {
    let Some((command, command_args)) = args.split_first() else {
        return Err(DifferentialError::usage("missing differential command"));
    };

    match command.as_str() {
        "compare" | "replay" | "minimize" => parse_scenario_command(command, command_args),
        "verify-determinism" => parse_determinism_command(command_args),
        "fixture" => parse_fixture_command(command_args),
        unknown => Err(DifferentialError::usage(format!(
            "unknown differential command `{unknown}`"
        ))),
    }
}

pub(super) fn parse_scenario_command(
    command: &str,
    args: &[String],
) -> Result<RunnerInvocation, DifferentialError> {
    let options = parse_options(args)?;
    require_exact_options(&options, &["--scenario", "--preset", "--session-profile"])?;
    let scenario = require_allowed(&options, "--scenario", &ALLOWED_SCENARIOS)?;
    let preset = require_allowed(&options, "--preset", &ALLOWED_PRESETS)?;
    let profile = require_allowed(&options, "--session-profile", &ALLOWED_PROFILES)?;
    let math_probe = if matches!(scenario, "math-probes" | "collision-probes" | "rigid-world") {
        let sanitizer_rigid_compare =
            scenario == "rigid-world" && command == "compare" && preset == "oracle-asan-ubsan";
        let shape_is_reviewed = profile == "one-shot"
            && (MATH_PROBE_PRESETS.contains(&preset) || sanitizer_rigid_compare);
        let action_is_reviewed = scenario == "rigid-world" || command != "minimize";
        if !shape_is_reviewed || !action_is_reviewed {
            return Err(DifferentialError::usage(
                "fixed evidence scenarios support only their reviewed one-shot debug or release shape",
            ));
        }
        Some(MathProbeInvocation {
            kind: match scenario {
                "math-probes" => ProbeKind::Math,
                "collision-probes" => ProbeKind::Collision,
                "rigid-world" => ProbeKind::Rigid,
                _ => unreachable!("closed fixed evidence scenario"),
            },
            action: match command {
                "compare" => MathProbeAction::Compare,
                "replay" => MathProbeAction::Replay,
                "minimize" => MathProbeAction::Minimize,
                _ => unreachable!("closed scenario command"),
            },
            preset: preset.to_owned(),
            runs: 1,
        })
    } else {
        None
    };

    Ok(RunnerInvocation {
        arguments: option_arguments(
            &[command],
            &[
                ("--scenario", scenario),
                ("--preset", preset),
                ("--session-profile", profile),
            ],
        ),
        oracle_dependent: true,
        math_probe,
    })
}

pub(super) fn parse_determinism_command(
    args: &[String],
) -> Result<RunnerInvocation, DifferentialError> {
    let options = parse_options(args)?;
    require_exact_options(&options, &["--scenario", "--preset", "--runs"])?;
    let scenario = require_allowed(
        &options,
        "--scenario",
        &["math-probes", "collision-probes", "rigid-world"],
    )?;
    let preset = require_allowed(&options, "--preset", &MATH_PROBE_PRESETS)?;
    let runs = require_allowed(&options, "--runs", &["2"])?;

    Ok(RunnerInvocation {
        arguments: option_arguments(
            &["verify-determinism"],
            &[
                ("--scenario", scenario),
                ("--preset", preset),
                ("--runs", runs),
            ],
        ),
        oracle_dependent: true,
        math_probe: Some(MathProbeInvocation {
            kind: match scenario {
                "math-probes" => ProbeKind::Math,
                "collision-probes" => ProbeKind::Collision,
                "rigid-world" => ProbeKind::Rigid,
                _ => unreachable!("closed determinism scenario"),
            },
            action: MathProbeAction::VerifyDeterminism,
            preset: preset.to_owned(),
            runs: 2,
        }),
    })
}

pub(super) fn parse_fixture_command(
    args: &[String],
) -> Result<RunnerInvocation, DifferentialError> {
    let Some((action, action_args)) = args.split_first() else {
        return Err(DifferentialError::usage("missing fixture action"));
    };
    let options = parse_options(action_args)?;

    match action.as_str() {
        "stage" => {
            require_exact_options(
                &options,
                &[
                    "--artifact-id",
                    "--artifact-kind",
                    "--preset",
                    "--scenario",
                    "--session-profile",
                ],
            )?;
            let scenario = require_allowed(&options, "--scenario", &ALLOWED_SCENARIOS)?;
            let preset = require_allowed(&options, "--preset", &ALLOWED_PRESETS)?;
            let profile = require_allowed(&options, "--session-profile", &ALLOWED_PROFILES)?;
            let artifact_kind =
                require_allowed(&options, "--artifact-kind", &ALLOWED_ARTIFACT_KINDS)?;
            let artifact_id = required_option(&options, "--artifact-id")?;
            Ok(RunnerInvocation {
                arguments: option_arguments(
                    &["fixture", "stage"],
                    &[
                        ("--scenario", scenario),
                        ("--preset", preset),
                        ("--session-profile", profile),
                        ("--artifact-kind", artifact_kind),
                        ("--artifact-id", artifact_id),
                    ],
                ),
                oracle_dependent: true,
                math_probe: None,
            })
        }
        "review" => {
            require_exact_options(
                &options,
                &[
                    "--artifact-id",
                    "--review-status",
                    "--reviewed-at",
                    "--reviewer",
                ],
            )?;
            let artifact_id = required_option(&options, "--artifact-id")?;
            let reviewer = required_option(&options, "--reviewer")?;
            let reviewed_at = required_option(&options, "--reviewed-at")?;
            let review_status =
                require_allowed(&options, "--review-status", &ALLOWED_REVIEW_STATUSES)?;
            Ok(RunnerInvocation {
                arguments: option_arguments(
                    &["fixture", "review"],
                    &[
                        ("--artifact-id", artifact_id),
                        ("--reviewer", reviewer),
                        ("--reviewed-at", reviewed_at),
                        ("--review-status", review_status),
                    ],
                ),
                oracle_dependent: false,
                math_probe: None,
            })
        }
        "promote" => {
            require_exact_options(&options, &["--artifact-id"])?;
            let artifact_id = required_option(&options, "--artifact-id")?;
            Ok(RunnerInvocation {
                arguments: option_arguments(
                    &["fixture", "promote"],
                    &[("--artifact-id", artifact_id)],
                ),
                oracle_dependent: false,
                math_probe: None,
            })
        }
        unknown => Err(DifferentialError::usage(format!(
            "unknown fixture action `{unknown}`"
        ))),
    }
}
