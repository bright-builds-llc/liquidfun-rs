use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args();
    let program = args.next().unwrap_or_default();
    let tool = Path::new(&program)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default();
    let args: Vec<String> = args.collect();

    if tool.contains("git") {
        return run_git(&args);
    }
    if tool.contains("samply") {
        return run_samply(&args);
    }
    if tool.contains("cargo") {
        return run_cargo(&args);
    }
    if tool.contains("cmake") {
        return run_cmake(&args);
    }
    if tool.contains("ninja") {
        println!("1.13.2");
        return ExitCode::SUCCESS;
    }
    if tool.contains("cxx") {
        println!("clang version 22.1.8");
        return ExitCode::SUCCESS;
    }
    if tool.contains("playground-dam-break-bench") {
        return print_bench_sample("pinned_cpp", 1.0, "AppleClang 17.0.0", &args);
    }

    eprintln!("unknown fake tool `{tool}`");
    ExitCode::FAILURE
}

fn run_cargo(args: &[String]) -> ExitCode {
    if env::var_os("LIQUIDFUN_TEST_ASSERT_PACKAGE_ISOLATION").is_some() {
        let current = env::current_dir().unwrap_or_default();
        let has_forbidden_directory = [
            "third_party",
            "reference",
            "tools",
            "testbed",
            "crates/liquidfun-testbed",
        ]
        .iter()
        .any(|relative| current.join(relative).exists());
        let has_display = env::var_os("DISPLAY").is_some()
            || env::var_os("WAYLAND_DISPLAY").is_some()
            || env::var_os("MIR_SOCKET").is_some()
            || env::var_os("XDG_RUNTIME_DIR").is_some()
            || env::var_os("LIQUIDFUN_XTASK_ROOT").is_some()
            || env::var_os("LIQUIDFUN_XTASK_TEST_PACKAGE_ARCHIVE").is_some();
        if has_forbidden_directory || has_display {
            eprintln!("package build/test environment was not isolated");
            return ExitCode::FAILURE;
        }
    }
    if let Some(marker) = env::var_os("LIQUIDFUN_TEST_CARGO_MARKER")
        && std::fs::write(marker, args.join(" ")).is_err()
    {
        return ExitCode::FAILURE;
    }
    if args.iter().any(|argument| argument == "run")
        && args.iter().any(|argument| argument == "dam-break-bench")
        && args.iter().any(|argument| argument.contains("dhat-heap"))
    {
        return run_dhat_heap_bench(args);
    }
    if args.iter().any(|argument| argument == "--profile")
        && args.iter().any(|argument| argument == "profiling")
    {
        return install_profiling_bench();
    }
    if args.iter().any(|argument| argument == "run")
        && args.iter().any(|argument| argument == "dam-break-timers")
    {
        return print_timer_sample(args);
    }
    if args.iter().any(|argument| argument == "run")
        && args.iter().any(|argument| argument == "playground-scene-spot")
    {
        return print_scene_spot_sample(args);
    }
    if args.iter().any(|argument| argument == "run")
        && args.iter().any(|argument| argument == "dam-break-bench")
    {
        return print_bench_sample("native_rust", 300.0, "rustc 1.97.0", args);
    }
    ExitCode::SUCCESS
}

fn run_dhat_heap_bench(args: &[String]) -> ExitCode {
    let Some(path) = env::var_os("LIQUIDFUN_DHAT_HEAP_FILE") else {
        eprintln!("LIQUIDFUN_DHAT_HEAP_FILE is unset");
        return ExitCode::FAILURE;
    };
    const DUMP_BYTES: &[u8] = b"fake-dhat-heap-json-dump";
    if std::fs::write(&path, DUMP_BYTES).is_err() {
        eprintln!("failed to write {}", PathBuf::from(&path).display());
        return ExitCode::FAILURE;
    }
    print_bench_sample("native_rust", 300.0, "rustc 1.97.0", args)
}

fn install_profiling_bench() -> ExitCode {
    let file_name = if cfg!(windows) {
        "dam-break-bench.exe"
    } else {
        "dam-break-bench"
    };
    let target_dir = env::var_os("CARGO_TARGET_DIR").map_or_else(
        || env::current_dir().unwrap_or_default().join("target"),
        PathBuf::from,
    );
    let dir = target_dir.join("profiling");
    if std::fs::create_dir_all(&dir).is_err() {
        eprintln!("failed to create {}", dir.display());
        return ExitCode::FAILURE;
    }
    let destination = dir.join(file_name);
    let Ok(current) = env::current_exe() else {
        eprintln!("failed to locate fake cargo executable");
        return ExitCode::FAILURE;
    };
    if std::fs::copy(current, &destination).is_err() {
        eprintln!("failed to install {}", destination.display());
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run_samply(args: &[String]) -> ExitCode {
    if args.iter().any(|argument| argument == "--version") {
        println!("samply 0.13.1");
        return ExitCode::SUCCESS;
    }
    if args.first().map(String::as_str) != Some("record") {
        eprintln!("unsupported fake samply arguments: {args:?}");
        return ExitCode::FAILURE;
    }
    if let Some(marker) = env::var_os("LIQUIDFUN_TEST_SAMPLY_MARKER")
        && std::fs::write(marker, args.join(" ")).is_err()
    {
        return ExitCode::FAILURE;
    }
    if !args.iter().any(|argument| argument == "--save-only") {
        eprintln!("fake samply record requires --save-only");
        return ExitCode::FAILURE;
    }
    let Some(output_index) = args.iter().position(|argument| argument == "-o") else {
        return ExitCode::FAILURE;
    };
    let Some(output) = args.get(output_index + 1) else {
        eprintln!("missing value for -o");
        return ExitCode::FAILURE;
    };
    const PROFILE_BYTES: &[u8] = b"fake-samply-json-gz";
    if std::fs::write(output, PROFILE_BYTES).is_err() {
        eprintln!("failed to write {output}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run_git(args: &[String]) -> ExitCode {
    let revision = env::var("LIQUIDFUN_TEST_REVISION").unwrap_or_default();
    let remote_url = env::var("LIQUIDFUN_TEST_REMOTE_URL").unwrap_or_default();

    if args.iter().any(|argument| argument == "ls-tree") {
        println!("160000 commit {revision}\tthird_party/liquidfun");
        return ExitCode::SUCCESS;
    }
    if args.iter().any(|argument| argument == "rev-parse") {
        println!("{revision}");
        return ExitCode::SUCCESS;
    }
    if args.iter().any(|argument| argument == "cat-file") {
        let expected_object = format!("{revision}^{{commit}}");
        if args.last() == Some(&expected_object) {
            return ExitCode::SUCCESS;
        }
        eprintln!("unknown generator revision");
        return ExitCode::FAILURE;
    }
    if args.iter().any(|argument| argument == "status") {
        if env::var_os("LIQUIDFUN_TEST_DIRTY").is_some() {
            println!(" M liquidfun/Box2D/dirty.cpp");
        }
        return ExitCode::SUCCESS;
    }
    if args.iter().any(|argument| argument == "remote") {
        println!("{remote_url}");
        return ExitCode::SUCCESS;
    }

    eprintln!("unsupported fake git arguments: {args:?}");
    ExitCode::FAILURE
}

fn run_cmake(args: &[String]) -> ExitCode {
    if args == ["--version"] {
        println!("cmake version 4.3.3");
        return ExitCode::SUCCESS;
    }
    if env::var_os("LIQUIDFUN_TEST_CMAKE_FAIL_STDOUT").is_some() {
        println!("simulated compiler failure on stdout");
        return ExitCode::from(42);
    }
    if env::var_os("LIQUIDFUN_TEST_CMAKE_FAIL").is_some() {
        eprintln!("simulated cmake failure");
        return ExitCode::from(42);
    }
    if let Some(marker) = env::var_os("LIQUIDFUN_TEST_CMAKE_MARKER")
        && std::fs::write(marker, args.join("\n")).is_err()
    {
        return ExitCode::FAILURE;
    }

    println!("simulated cmake invocation: {args:?}");
    ExitCode::SUCCESS
}

fn print_bench_sample(engine: &str, wall_ms: f64, compiler: &str, args: &[String]) -> ExitCode {
    let warmup_steps = match parse_u32_flag(args, "--warmup", 60) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let measured_steps = match parse_u32_flag(args, "--steps", 600) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let ms_per_step = if measured_steps == 0 {
        0.0
    } else {
        wall_ms / f64::from(measured_steps)
    };
    let steps_per_s = if ms_per_step <= 0.0 {
        0.0
    } else {
        1000.0 / ms_per_step
    };
    let realtime_factor = if wall_ms <= 0.0 {
        0.0
    } else {
        (f64::from(measured_steps) / 60.0) / (wall_ms / 1000.0)
    };
    println!(
        "{{\"engine\":\"{engine}\",\"particles\":1920,\"warmup_steps\":{warmup_steps},\"measured_steps\":{measured_steps},\"wall_ms\":{wall_ms},\"ms_per_step\":{ms_per_step},\"steps_per_s\":{steps_per_s},\"realtime_factor\":{realtime_factor},\"compiler\":\"{compiler}\"}}"
    );
    ExitCode::SUCCESS
}

fn print_scene_spot_sample(args: &[String]) -> ExitCode {
    let warmup_steps = match parse_u32_flag(args, "--warmup", 60) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let measured_steps = match parse_u32_flag(args, "--steps", 120) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let scenes = [
        "fountain",
        "float-or-sink",
        "color-mixer",
        "jelly-drop",
        "water-wheel",
    ];
    for (index, scene) in scenes.iter().enumerate() {
        let start_particles = 100 + index * 10;
        let end_particles = start_particles + 5;
        println!(
            "{{\"scene\":\"{scene}\",\"warmup_steps\":{warmup_steps},\"measured_steps\":{measured_steps},\"start_particles\":{start_particles},\"end_particles\":{end_particles},\"wall_ms\":1.0,\"ms_per_step\":1.0,\"timed_out\":false}}"
        );
    }
    ExitCode::SUCCESS
}

fn print_timer_sample(args: &[String]) -> ExitCode {
    let warmup_steps = match parse_u32_flag(args, "--warmup", 60) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let measured_steps = match parse_u32_flag(args, "--steps", 600) {
        Ok(value) => value,
        Err(code) => return code,
    };
    println!(
        "{{\"kind\":\"step_profiled_parents\",\"schema\":\"phase12-profile-v1\",\"not_timing_authority\":true,\"particles\":1920,\"warmup_steps\":{warmup_steps},\"measured_steps\":{measured_steps},\"parents\":{{\"contact_update\":{{\"wall_ms\":0.0}},\"rigid_solve\":{{\"wall_ms\":1.0}},\"continuous_solve\":{{\"wall_ms\":0.0}},\"particle_prepare\":{{\"wall_ms\":1.0}},\"particle_solve\":{{\"wall_ms\":1.0}},\"finalize\":{{\"wall_ms\":0.0}}}}}}"
    );
    ExitCode::SUCCESS
}

fn parse_u32_flag(args: &[String], flag: &str, default: u32) -> Result<u32, ExitCode> {
    let Some(index) = args.iter().position(|argument| argument == flag) else {
        return Ok(default);
    };
    let Some(value) = args.get(index + 1) else {
        eprintln!("missing value for {flag}");
        return Err(ExitCode::FAILURE);
    };
    value.parse().map_err(|_| {
        eprintln!("invalid {flag} value `{value}`");
        ExitCode::FAILURE
    })
}
