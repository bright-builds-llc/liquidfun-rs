//! Fake-tool coverage for unprofiled Dam Break pair persistence.

use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

const REPOSITORY: &str = "https://github.com/google/liquidfun.git";
const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
const ADAPTER_SOURCES: [&str; 9] = [
    "tools/reference/src/main.cpp",
    "tools/reference/src/protocol.cpp",
    "tools/reference/src/protocol_bits.cpp",
    "tools/reference/src/protocol.hpp",
    "tools/reference/src/oracle_adapter.cpp",
    "tools/reference/src/oracle_adapter.hpp",
    "tools/reference/src/build_identity.hpp.in",
    "tools/reference/vendor/nlohmann/json.hpp",
    "tools/reference/CMakeLists.txt",
];
const ADAPTER_INPUT_MANIFEST: &str = "tools/reference/adapter-inputs.txt";
const STAMP_UNIX: &str = "1000000000";
const FIRST_STAMP: &str = "2001-09-09T01-46-40Z";
const SECOND_STAMP: &str = "2001-09-09T01-46-41Z";

static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);
static FAKE_TOOLS: OnceLock<Result<FakeTools, String>> = OnceLock::new();

type TestResult = Result<(), Box<dyn Error>>;

#[derive(Debug)]
struct FakeTools {
    git: PathBuf,
    cmake: PathBuf,
    ninja: PathBuf,
    cxx: PathBuf,
    cargo: PathBuf,
    base: PathBuf,
}

#[derive(Debug)]
struct RepositoryFixture {
    root: PathBuf,
    cmake_marker: PathBuf,
}

impl RepositoryFixture {
    fn new() -> io::Result<Self> {
        let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let root = workspace_root().join(format!(
            "target/xtask-test-fixtures/playground-cli-{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("reference"))?;
        fs::create_dir_all(root.join("third_party/liquidfun"))?;
        fs::create_dir_all(root.join("tools/reference/src"))?;
        fs::create_dir_all(root.join("crates/liquidfun"))?;
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nresolver = \"3\"\nmembers = [\"crates/liquidfun\"]\n",
        )?;
        fs::write(
            root.join("crates/liquidfun/Cargo.toml"),
            "[package]\nname = \"liquidfun\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
        )?;

        let cmake_marker = root.join("cmake-arguments.txt");
        let fixture = Self { root, cmake_marker };
        fixture.write_lock()?;
        fixture.write_adapter_sources()?;
        fs::write(
            fixture.root.join(".gitmodules"),
            format!(
                "[submodule \"third_party/liquidfun\"]\n\tpath = third_party/liquidfun\n\turl = {REPOSITORY}\n"
            ),
        )?;
        fixture.install_cpp_bench()?;
        Ok(fixture)
    }

    fn write_lock(&self) -> io::Result<()> {
        fs::write(
            self.root.join("reference/upstream-lock.toml"),
            format!(
                "schema_version = 1\nrepository = \"{REPOSITORY}\"\nrevision = \"{REVISION}\"\nsubmodule_path = \"third_party/liquidfun\"\n"
            ),
        )
    }

    fn write_adapter_sources(&self) -> io::Result<()> {
        for relative_path in ADAPTER_SOURCES {
            if let Some(parent) = self.root.join(relative_path).parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(self.root.join(relative_path), format!("{relative_path}\n"))?;
        }
        fs::write(
            self.root.join(ADAPTER_INPUT_MANIFEST),
            format!("{}\n", ADAPTER_SOURCES.join("\n")),
        )?;
        Ok(())
    }

    fn install_cpp_bench(&self) -> io::Result<()> {
        let tools = fake_tools()?;
        let dir = self.root.join("target/reference/oracle-release");
        fs::create_dir_all(&dir)?;
        fs::copy(
            &tools.base,
            dir.join(executable_name("playground-dam-break-bench")),
        )?;
        Ok(())
    }

    fn cmake_arguments(&self) -> io::Result<Vec<String>> {
        Ok(fs::read_to_string(&self.cmake_marker)?
            .lines()
            .map(str::to_owned)
            .collect())
    }

    fn command(&self) -> io::Result<Command> {
        let tools = fake_tools()?;
        let mut command = Command::new(env!("CARGO_BIN_EXE_xtask"));
        command
            .current_dir(&self.root)
            .env("LIQUIDFUN_XTASK_GIT", &tools.git)
            .env("LIQUIDFUN_XTASK_CMAKE", &tools.cmake)
            .env("LIQUIDFUN_XTASK_NINJA", &tools.ninja)
            .env("LIQUIDFUN_XTASK_CXX", &tools.cxx)
            .env("LIQUIDFUN_XTASK_CARGO", &tools.cargo)
            .env("LIQUIDFUN_XTASK_STAMP_UNIX", STAMP_UNIX)
            .env("LIQUIDFUN_TEST_REVISION", REVISION)
            .env("LIQUIDFUN_TEST_REMOTE_URL", REPOSITORY)
            .env("LIQUIDFUN_TEST_CMAKE_MARKER", &self.cmake_marker);
        Ok(command)
    }

    fn pair_json(&self, stamp: &str) -> PathBuf {
        self.root
            .join("target/dam-break-perf")
            .join(stamp)
            .join("pair.json")
    }

    fn pair_md(&self, stamp: &str) -> PathBuf {
        self.root
            .join("target/dam-break-perf")
            .join(stamp)
            .join("pair.md")
    }

    fn cleanup(self) -> io::Result<()> {
        fs::remove_dir_all(self.root)
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .components()
        .collect()
}

fn fake_tools() -> io::Result<&'static FakeTools> {
    let result = FAKE_TOOLS.get_or_init(compile_fake_tools);
    match result {
        Ok(tools) => Ok(tools),
        Err(message) => Err(io::Error::other(message.clone())),
    }
}

fn compile_fake_tools() -> Result<FakeTools, String> {
    let output_dir = workspace_root().join(format!(
        "target/xtask-test-tools/playground-cli-{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).map_err(|error| error.to_string())?;
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fake_upstream_tool.rs");
    let base = output_dir.join(executable_name("fake-base"));
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"));
    let output = Command::new(rustc)
        .arg(&source)
        .arg("--edition=2024")
        .arg("-o")
        .arg(&base)
        .output()
        .map_err(|error| format!("failed to compile {}: {error}", source.display()))?;
    if !output.status.success() {
        return Err(format!(
            "failed to compile {}: {}",
            source.display(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(FakeTools {
        git: copy_fake_tool(&base, &output_dir, "fake-git")?,
        cmake: copy_fake_tool(&base, &output_dir, "fake-cmake")?,
        ninja: copy_fake_tool(&base, &output_dir, "fake-ninja")?,
        cxx: copy_fake_tool(&base, &output_dir, "fake-cxx")?,
        cargo: copy_fake_tool(&base, &output_dir, "fake-cargo")?,
        base,
    })
}

fn copy_fake_tool(base: &Path, output_dir: &Path, name: &str) -> Result<PathBuf, String> {
    let destination = output_dir.join(executable_name(name));
    fs::copy(base, &destination).map_err(|error| error.to_string())?;
    Ok(destination)
}

fn executable_name(stem: &str) -> String {
    format!("{stem}{}", env::consts::EXE_SUFFIX)
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn run_pair(fixture: &RepositoryFixture) -> io::Result<Output> {
    let mut command = fixture.command()?;
    command.args([
        "playground",
        "dam-break-bench",
        "--warmup",
        "0",
        "--steps",
        "1",
    ]);
    command.output()
}

#[test]
fn dam_break_bench_persists_unprofiled_pair_without_profile_cmake_flags() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;

    // Act
    let output = run_pair(&fixture)?;

    // Assert
    assert!(
        output.status.success(),
        "stderr: {}\nstdout: {}",
        stderr(&output),
        stdout(&output)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.pair_json(FIRST_STAMP))?)?;
    assert_eq!(report["kind"], "unprofiled_pair");
    assert_eq!(report["timing_authority"], "unprofiled_wall_clock");
    assert_eq!(report["rust_over_cpp_ratio"].as_f64(), Some(300.0));
    assert_eq!(report["rust"]["wall_ms"].as_f64(), Some(300.0));
    assert_eq!(report["cpp"]["wall_ms"].as_f64(), Some(1.0));
    let markdown = fs::read_to_string(fixture.pair_md(FIRST_STAMP))?;
    assert!(markdown.contains("Rust/C++"));
    assert!(markdown.contains("Unreviewed local playground Dam Break sample"));
    assert!(stdout(&output).contains("Unreviewed local playground Dam Break sample"));
    assert!(
        stdout(&output).contains(&markdown),
        "stdout should include the pair.md Markdown table"
    );
    let cmake_args = fixture.cmake_arguments()?;
    assert!(cmake_args.iter().any(|argument| argument == "--target"));
    assert!(
        cmake_args
            .iter()
            .any(|argument| argument == "playground-dam-break-bench")
    );
    assert!(!cmake_args.iter().any(|argument| argument == "-g"));
    assert!(
        !cmake_args
            .iter()
            .any(|argument| argument.contains("REFERENCE_PROFILE_DEBUG_INFO"))
    );
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn second_pair_in_the_same_unix_second_mints_a_new_stamp() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let first = run_pair(&fixture)?;
    assert!(first.status.success(), "{}", stderr(&first));
    let first_json = fs::read(fixture.pair_json(FIRST_STAMP))?;

    // Act
    let second = run_pair(&fixture)?;

    // Assert
    assert!(second.status.success(), "{}", stderr(&second));
    let first_json_after = fs::read(fixture.pair_json(FIRST_STAMP))?;
    assert_eq!(first_json, first_json_after);
    assert!(fixture.pair_json(SECOND_STAMP).is_file());
    assert_ne!(fs::read(fixture.pair_json(SECOND_STAMP))?, first_json_after);
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn justfile_keeps_the_one_line_dam_break_bench_alias() {
    // Arrange
    let justfile = include_str!("../../../justfile");

    // Act
    let recipe_start = justfile
        .find("playground-dam-break-bench:")
        .expect("justfile should contain the playground Dam Break recipe");
    let recipe = justfile[recipe_start..]
        .split("\n\n")
        .next()
        .expect("recipe should end at a blank line");

    // Assert
    assert_eq!(
        recipe,
        "playground-dam-break-bench:\n    cargo xtask playground dam-break-bench"
    );
    assert!(!justfile.contains("samply"));
    assert!(!justfile.to_ascii_lowercase().contains("cmake"));
}
