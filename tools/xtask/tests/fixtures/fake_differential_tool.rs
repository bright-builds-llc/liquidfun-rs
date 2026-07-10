use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    let Some(marker) = env::var_os("LIQUIDFUN_TEST_DIFFERENTIAL_MARKER") else {
        eprintln!("missing LIQUIDFUN_TEST_DIFFERENTIAL_MARKER");
        return ExitCode::FAILURE;
    };

    if let Err(error) = std::fs::write(marker, arguments.join("\n")) {
        eprintln!("failed to record differential arguments: {error}");
        return ExitCode::FAILURE;
    }

    if env::var_os("LIQUIDFUN_TEST_DIFFERENTIAL_FAIL").is_some() {
        eprintln!("simulated differential failure");
        return ExitCode::from(42);
    }

    println!("simulated differential invocation");
    ExitCode::SUCCESS
}
