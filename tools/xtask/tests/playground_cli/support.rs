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
pub(super) const STAMP_UNIX: &str = "1000000000";
pub(super) const FIRST_STAMP: &str = "2001-09-09T01-46-40Z";
pub(super) const SECOND_STAMP: &str = "2001-09-09T01-46-41Z";

static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);
static FAKE_TOOLS: OnceLock<Result<FakeTools, String>> = OnceLock::new();

pub(super) type TestResult = Result<(), Box<dyn Error>>;

#[derive(Debug)]
pub(super) struct FakeTools {
    git: PathBuf,
    cmake: PathBuf,
    ninja: PathBuf,
    cxx: PathBuf,
    cargo: PathBuf,
    samply: PathBuf,
    base: PathBuf,
}

#[derive(Debug)]
pub(super) struct RepositoryFixture {
    pub(super) root: PathBuf,
    cmake_marker: PathBuf,
}

impl RepositoryFixture {
    pub(super) fn new() -> io::Result<Self> {
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

    pub(super) fn cmake_arguments(&self) -> io::Result<Vec<String>> {
        Ok(fs::read_to_string(&self.cmake_marker)?
            .lines()
            .map(str::to_owned)
            .collect())
    }

    pub(super) fn command(&self) -> io::Result<Command> {
        let tools = fake_tools()?;
        let mut command = Command::new(env!("CARGO_BIN_EXE_xtask"));
        command
            .current_dir(&self.root)
            .env("LIQUIDFUN_XTASK_GIT", &tools.git)
            .env("LIQUIDFUN_XTASK_CMAKE", &tools.cmake)
            .env("LIQUIDFUN_XTASK_NINJA", &tools.ninja)
            .env("LIQUIDFUN_XTASK_CXX", &tools.cxx)
            .env("LIQUIDFUN_XTASK_CARGO", &tools.cargo)
            .env("LIQUIDFUN_XTASK_SAMPLY", &tools.samply)
            .env("LIQUIDFUN_XTASK_STAMP_UNIX", STAMP_UNIX)
            .env("CARGO_TARGET_DIR", self.root.join("target"))
            .env("LIQUIDFUN_TEST_REVISION", REVISION)
            .env("LIQUIDFUN_TEST_REMOTE_URL", REPOSITORY)
            .env("LIQUIDFUN_TEST_CMAKE_MARKER", &self.cmake_marker)
            .env("LIQUIDFUN_TEST_CARGO_MARKER", self.cargo_marker());
        Ok(command)
    }

    pub(super) fn cargo_marker(&self) -> PathBuf {
        self.root.join("cargo-arguments.txt")
    }

    pub(super) fn profile_gz(&self, stamp: &str) -> PathBuf {
        self.root
            .join("target/dam-break-perf")
            .join(stamp)
            .join("rust.json.gz")
    }

    pub(super) fn profile_identity(&self, stamp: &str) -> PathBuf {
        self.root
            .join("target/dam-break-perf")
            .join(stamp)
            .join("profile-identity.json")
    }

    pub(super) fn pair_json(&self, stamp: &str) -> PathBuf {
        self.root
            .join("target/dam-break-perf")
            .join(stamp)
            .join("pair.json")
    }

    pub(super) fn pair_md(&self, stamp: &str) -> PathBuf {
        self.root
            .join("target/dam-break-perf")
            .join(stamp)
            .join("pair.md")
    }

    pub(super) fn cleanup(self) -> io::Result<()> {
        fs::remove_dir_all(self.root)
    }
}

pub(super) fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .components()
        .collect()
}

pub(super) fn fake_tools() -> io::Result<&'static FakeTools> {
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
        samply: copy_fake_tool(&base, &output_dir, "fake-samply")?,
        base,
    })
}

fn copy_fake_tool(base: &Path, output_dir: &Path, name: &str) -> Result<PathBuf, String> {
    let destination = output_dir.join(executable_name(name));
    fs::copy(base, &destination).map_err(|error| error.to_string())?;
    Ok(destination)
}

pub(super) fn executable_name(stem: &str) -> String {
    format!("{stem}{}", env::consts::EXE_SUFFIX)
}

pub(super) fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub(super) fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

pub(super) fn run_pair(fixture: &RepositoryFixture) -> io::Result<Output> {
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

pub(super) fn run_profile(fixture: &RepositoryFixture) -> io::Result<Output> {
    let mut command = fixture.command()?;
    command.args([
        "playground",
        "dam-break-profile",
        "--warmup",
        "0",
        "--steps",
        "1",
    ]);
    command.output()
}

pub(super) fn rust_profiles(fixture: &RepositoryFixture) -> io::Result<Vec<PathBuf>> {
    let evidence_root = fixture.root.join("target/dam-break-perf");
    if !evidence_root.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for entry in fs::read_dir(evidence_root)? {
        let path = entry?.path().join("rust.json.gz");
        if path.is_file() {
            files.push(path);
        }
    }
    Ok(files)
}

pub(super) fn command_strings(identity: &serde_json::Value) -> Vec<String> {
    identity["command"]
        .as_array()
        .expect("profile-identity.json command should be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("command entries should be strings")
                .to_owned()
        })
        .collect()
}
