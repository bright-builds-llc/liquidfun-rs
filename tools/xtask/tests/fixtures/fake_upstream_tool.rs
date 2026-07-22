use std::env;
use std::path::Path;
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
    if tool.contains("cargo") {
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
        if let Some(marker) = env::var_os("LIQUIDFUN_TEST_CARGO_MARKER") {
            if std::fs::write(marker, args.join(" ")).is_err() {
                return ExitCode::FAILURE;
            }
        }
        return ExitCode::SUCCESS;
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

    eprintln!("unknown fake tool `{tool}`");
    ExitCode::FAILURE
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
