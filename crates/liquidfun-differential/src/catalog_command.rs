//! Renderer-free catalog discovery, controller execution, replay, and comparison.

use std::env;
use std::process::ExitCode;

use liquidfun_differential::{
    CatalogOracleSupervisor, CatalogRunOutcome, NativeCatalogBackend, OracleExecutable,
    SessionCommand, SessionController, SessionState, compare_catalog, execute_catalog_native,
    replay_catalog_exact_native,
};
use liquidfun_test_protocol::{
    CatalogDefinition, CatalogRunRequest, CatalogSlug, EvidenceTier, HarnessLimits, RequestId,
    ResolveRequest, RunProvenanceRequirements, ScenarioConsumer, Sha256Hex,
    encode_catalog_run_request_jsonl, resolve_catalog, reviewed_scenario_catalog,
};

mod parse;
mod render;

use parse::{
    CatalogCliError, CommandScript, ControllerCommand, ExecutionConfig, OutputMode, parse_options,
    require_shape, required,
};
use render::{InspectReport, RunReport, RunSnapshot, render_inspection, render_run};

const ORACLE_REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
const EXIT_PHYSICS_MISMATCH: u8 = 2;
const EXIT_HARNESS_FAILURE: u8 = 3;

pub(crate) fn run(args: &[String]) -> ExitCode {
    match execute(args) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("catalog/{}: {}", error.kind.as_str(), error.message);
            ExitCode::from(error.kind.exit_code())
        }
    }
}

fn execute(args: &[String]) -> Result<ExitCode, CatalogCliError> {
    let Some((action, tail)) = args.split_first() else {
        return Err(CatalogCliError::usage("missing catalog action"));
    };
    match action.as_str() {
        "list" => list(tail),
        "inspect" => inspect(tail),
        "run" => execute_native(tail),
        "replay" => replay(tail),
        "compare" => compare(tail),
        unknown => Err(CatalogCliError::usage(format!(
            "unknown catalog action `{unknown}`"
        ))),
    }
}

fn list(args: &[String]) -> Result<ExitCode, CatalogCliError> {
    if !args.is_empty() {
        return Err(CatalogCliError::usage(
            "catalog list does not accept arguments",
        ));
    }
    let catalog = catalog()?;
    for definition in catalog.definitions() {
        let metadata = definition.metadata().ok_or_else(|| {
            CatalogCliError::scenario("catalog definition has no reviewed metadata")
        })?;
        println!(
            "scenario: {} | title: {} | version: {} | seed: {} | visual: {}",
            definition.slug().as_str(),
            definition.display_title(),
            definition.scenario_version().get(),
            seed_contract(catalog.definitions(), definition)?,
            metadata.coverage().is_eligible(ScenarioConsumer::Visual)
        );
    }
    Ok(ExitCode::SUCCESS)
}

fn inspect(args: &[String]) -> Result<ExitCode, CatalogCliError> {
    let options = parse_options(args)?;
    require_shape(&options, &["--scenario", "--output"], &[])?;
    let output = OutputMode::parse(required(&options, "--output")?)?;
    let catalog = catalog()?;
    let definition = definition(&catalog, required(&options, "--scenario")?)?;
    let metadata = definition
        .metadata()
        .ok_or_else(|| CatalogCliError::scenario("catalog definition has no reviewed metadata"))?;
    let settings = metadata.default_settings();
    let report = InspectReport {
        record_kind: "catalog_inspection",
        scenario: definition.slug().as_str(),
        title: definition.display_title(),
        scenario_version: definition.scenario_version().get(),
        seed: seed_contract(catalog.definitions(), definition)?,
        timestep_bits: settings.timestep_bits().bits(),
        velocity_iterations: settings.velocity_iterations(),
        position_iterations: settings.position_iterations(),
        particle_iterations: settings.particle_iterations(),
        tags: metadata.tags().iter().map(CatalogSlug::as_str).collect(),
        visual_eligible: metadata.coverage().is_eligible(ScenarioConsumer::Visual),
    };
    render_inspection(output, &report)?;
    Ok(ExitCode::SUCCESS)
}

fn execute_native(args: &[String]) -> Result<ExitCode, CatalogCliError> {
    let config = ExecutionConfig::parse(args)?;
    let request = native_request(&config)?;
    let snapshot = execute_controller(&request, &config.commands)?;
    let report = RunReport::from_snapshot(&config, &request, snapshot, "not_requested");
    render_run(config.output, &report)?;
    Ok(ExitCode::SUCCESS)
}

fn replay(args: &[String]) -> Result<ExitCode, CatalogCliError> {
    let config = ExecutionConfig::parse(args)?;
    let request = native_request(&config)?;
    let first = execute_catalog_native(&request)
        .map_err(|error| CatalogCliError::harness(format!("{:?}", error.kind())))?;
    let bytes = encode_catalog_run_request_jsonl(&request, &HarnessLimits::phase2_default_v1())
        .map_err(|_| CatalogCliError::harness("request could not be encoded exactly"))?;
    let replayed = replay_catalog_exact_native(&bytes)
        .map_err(|error| CatalogCliError::harness(format!("{:?}", error.kind())))?;
    if first.canonical_checkpoint_bytes() != replayed.canonical_checkpoint_bytes() {
        return Err(CatalogCliError::harness(
            "D0 replay changed canonical semantic checkpoint bytes",
        ));
    }
    let report = RunReport::from_capture(&config, &request, &first, "d0_replay_exact");
    render_run(config.output, &report)?;
    Ok(ExitCode::SUCCESS)
}

fn compare(args: &[String]) -> Result<ExitCode, CatalogCliError> {
    let config = ExecutionConfig::parse(args)?;
    let resolved = resolve(&config)?;
    let executable = OracleExecutable::resolve(&repository_root()?, config.preset)
        .map_err(|_| CatalogCliError::oracle("pinned oracle executable is unavailable"))?;
    let mut supervisor = CatalogOracleSupervisor::new(executable, config.profile, ORACLE_REVISION);
    let identity = supervisor.discover_identity().map_err(|error| {
        CatalogCliError::oracle(format!("oracle startup failed: {:?}", error.kind()))
    })?;
    let request = request(
        resolved,
        identity.identity_sha256().clone(),
        supervisor.limits_profile_sha256(),
    )?;
    let native = execute_catalog_native(&request)
        .map_err(|error| CatalogCliError::harness(format!("{:?}", error.kind())))?;
    let oracle = supervisor.execute(&request).map_err(|error| {
        CatalogCliError::harness(format!("oracle execution failed: {:?}", error.kind()))
    })?;
    let outcome = compare_catalog(&native, oracle.capture())
        .map_err(|error| CatalogCliError::harness(format!("{:?}", error.kind())))?;
    match outcome {
        CatalogRunOutcome::Match(matched) => {
            let summary = format!("exact_or_within_policy:{}", matched.comparisons().len());
            let report = RunReport::from_capture(&config, &request, &native, &summary);
            render_run(config.output, &report)?;
            Ok(ExitCode::SUCCESS)
        }
        CatalogRunOutcome::PhysicsMismatch(mismatch) => {
            let summary = format!(
                "physics_mismatch:{}",
                mismatch.first_mismatch().checkpoint_id().as_str()
            );
            let report = RunReport::from_capture(&config, &request, &native, &summary);
            render_run(config.output, &report)?;
            Ok(ExitCode::from(EXIT_PHYSICS_MISMATCH))
        }
        CatalogRunOutcome::HarnessFailure(kind) => {
            eprintln!("catalog/harness-failure: {kind:?}");
            Ok(ExitCode::from(EXIT_HARNESS_FAILURE))
        }
    }
}

fn execute_controller(
    request: &CatalogRunRequest,
    script: &CommandScript,
) -> Result<RunSnapshot, CatalogCliError> {
    let mut backend = NativeCatalogBackend::new();
    backend.set_request_id(request.request_id().clone());
    let mut controller = SessionController::new(backend);
    submit(
        &mut controller,
        SessionCommand::Select {
            resolved: request.resolved().clone(),
        },
    )?;
    match script {
        CommandScript::Auto => {
            for checkpoint in request.resolved().checkpoints() {
                submit(&mut controller, SessionCommand::StepOnce)?;
                submit(
                    &mut controller,
                    SessionCommand::CaptureCheckpoint {
                        checkpoint_id: checkpoint.checkpoint_id().clone(),
                    },
                )?;
            }
        }
        CommandScript::Explicit(commands) => {
            for command in commands {
                apply_command(&mut controller, command)?;
            }
        }
    }
    Ok(RunSnapshot {
        state: controller.state(),
        logical_steps: controller.completed_logical_steps(),
        checkpoints: controller
            .captures()
            .iter()
            .map(|capture| capture.identity().checkpoint_id().as_str().to_owned())
            .collect(),
    })
}

fn apply_command(
    controller: &mut SessionController<NativeCatalogBackend>,
    command: &ControllerCommand,
) -> Result<(), CatalogCliError> {
    match command {
        ControllerCommand::Pause if controller.state() == SessionState::Running => {
            submit(controller, SessionCommand::Pause)
        }
        ControllerCommand::Resume if controller.state() == SessionState::ReadyPaused => {
            submit(controller, SessionCommand::Run)
        }
        ControllerCommand::Pause | ControllerCommand::Resume => Ok(()),
        ControllerCommand::Step => submit(controller, SessionCommand::StepOnce),
        ControllerCommand::Restart => submit(controller, SessionCommand::Restart),
        ControllerCommand::ScenarioAction(action_id) => submit(
            controller,
            SessionCommand::ApplyScenarioAction {
                action_id: action_id.clone(),
            },
        ),
        ControllerCommand::Capture(checkpoint_id) => submit(
            controller,
            SessionCommand::CaptureCheckpoint {
                checkpoint_id: checkpoint_id.clone(),
            },
        ),
    }
}

fn submit(
    controller: &mut SessionController<NativeCatalogBackend>,
    command: SessionCommand,
) -> Result<(), CatalogCliError> {
    let command_id = controller
        .next_command_id()
        .ok_or_else(|| CatalogCliError::script("controller command bound exhausted"))?;
    controller.submit(command_id, command).map_err(|error| {
        CatalogCliError::script(format!("controller rejected command: {:?}", error.kind()))
    })?;
    Ok(())
}

fn native_request(config: &ExecutionConfig) -> Result<CatalogRunRequest, CatalogCliError> {
    request(
        resolve(config)?,
        Sha256Hex::new("0".repeat(64))
            .map_err(|_| CatalogCliError::harness("internal identity construction failed"))?,
        HarnessLimits::phase2_default_v1().profile_sha256(),
    )
}

fn request(
    resolved: liquidfun_test_protocol::ResolvedScenario,
    identity: Sha256Hex,
    limits: Sha256Hex,
) -> Result<CatalogRunRequest, CatalogCliError> {
    let hash = resolved.identity().content_sha256().as_str();
    let request_id = RequestId::new(format!("catalog-{}", &hash[..16]))
        .map_err(|_| CatalogCliError::harness("request identity construction failed"))?;
    CatalogRunRequest::new(
        request_id,
        resolved,
        RunProvenanceRequirements::new(identity, limits, EvidenceTier::D3Exploratory),
    )
    .map_err(|_| CatalogCliError::harness("resolved request validation failed"))
}

fn resolve(
    config: &ExecutionConfig,
) -> Result<liquidfun_test_protocol::ResolvedScenario, CatalogCliError> {
    let catalog = catalog()?;
    definition(&catalog, config.scenario.as_str())?;
    resolve_catalog(
        catalog.definitions(),
        &ResolveRequest::new(config.scenario.clone(), config.maybe_seed, config.settings),
    )
    .map_err(|error| CatalogCliError::scenario(format!("{:?}", error.kind())))
}

fn catalog() -> Result<liquidfun_test_protocol::ScenarioCatalog, CatalogCliError> {
    reviewed_scenario_catalog()
        .map_err(|error| CatalogCliError::scenario(format!("{:?}", error.kind())))
}

fn definition<'a>(
    catalog: &'a liquidfun_test_protocol::ScenarioCatalog,
    value: &str,
) -> Result<&'a CatalogDefinition, CatalogCliError> {
    let slug = CatalogSlug::new(value.to_owned())
        .map_err(|_| CatalogCliError::scenario("scenario must be a stable catalog slug"))?;
    catalog
        .definitions()
        .iter()
        .find(|definition| definition.slug() == &slug)
        .ok_or_else(|| CatalogCliError::scenario(format!("unknown scenario slug `{value}`")))
}

fn seed_contract(
    definitions: &[CatalogDefinition],
    definition: &CatalogDefinition,
) -> Result<&'static str, CatalogCliError> {
    let settings = definition
        .metadata()
        .ok_or_else(|| CatalogCliError::scenario("catalog metadata is absent"))?
        .default_settings();
    match resolve_catalog(
        definitions,
        &ResolveRequest::new(definition.slug().clone(), None, settings),
    ) {
        Ok(_) => Ok("none"),
        Err(error) if error.kind() == liquidfun_test_protocol::CatalogErrorKind::SeedRequired => {
            Ok("required")
        }
        Err(error) => Err(CatalogCliError::scenario(format!("{:?}", error.kind()))),
    }
}

fn repository_root() -> Result<std::path::PathBuf, CatalogCliError> {
    let current = env::current_dir()
        .map_err(|error| CatalogCliError::harness(format!("current directory: {error}")))?;
    current
        .ancestors()
        .find(|path| path.join("Cargo.toml").is_file() && path.join("reference").is_dir())
        .map(std::path::Path::to_path_buf)
        .ok_or_else(|| CatalogCliError::harness("repository root is unavailable"))
}
