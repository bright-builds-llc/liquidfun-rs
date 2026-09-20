//! Command-level coverage for the pinned upstream oracle workflow.

use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};

#[path = "upstream_cli/build.rs"]
mod build;
#[path = "upstream_cli/configure.rs"]
mod configure;
#[path = "upstream_cli/failures.rs"]
mod failures;
#[path = "upstream_cli/verify.rs"]
mod verify;

const REPOSITORY: &str = "https://github.com/google/liquidfun.git";
const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
const WRONG_REVISION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
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

static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);
static FAKE_TOOLS: OnceLock<Result<FakeTools, String>> = OnceLock::new();

type TestResult = Result<(), Box<dyn Error>>;

#[derive(Debug)]
struct FakeTools {
    git: PathBuf,
    cmake: PathBuf,
    ninja: PathBuf,
    cxx: PathBuf,
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
            "target/xtask-test-fixtures/{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("reference"))?;
        fs::create_dir_all(root.join("third_party/liquidfun"))?;
        fs::create_dir_all(root.join("tools/reference/src"))?;

        let cmake_marker = root.join("cmake-arguments.txt");
        let fixture = Self { root, cmake_marker };
        fixture.write_lock(REVISION)?;
        fixture.write_adapter_sources()?;
        fs::write(
            fixture.root.join(".gitmodules"),
            format!(
                "[submodule \"third_party/liquidfun\"]\n\tpath = third_party/liquidfun\n\turl = {REPOSITORY}\n"
            ),
        )?;
        Ok(fixture)
    }

    fn write_lock(&self, revision: &str) -> io::Result<()> {
        fs::write(
            self.root.join("reference/upstream-lock.toml"),
            format!(
                "schema_version = 1\nrepository = \"{REPOSITORY}\"\nrevision = \"{revision}\"\nsubmodule_path = \"third_party/liquidfun\"\n"
            ),
        )
    }

    fn remove_submodule(&self) -> io::Result<()> {
        fs::remove_dir_all(self.root.join("third_party/liquidfun"))
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

    fn adapter_source_digest(&self) -> io::Result<String> {
        let mut digest_input = Sha256::new();
        let manifest = fs::read_to_string(self.root.join(ADAPTER_INPUT_MANIFEST))?;
        for relative_path in manifest.lines() {
            let bytes = fs::read(self.root.join(relative_path))?;
            let source_digest = Sha256::digest(bytes);
            digest_input.update(relative_path.as_bytes());
            digest_input.update(b"=");
            digest_input.update(format!("{source_digest:x}").as_bytes());
            digest_input.update(b"\n");
        }
        Ok(format!("{:x}", digest_input.finalize()))
    }

    fn overwrite_adapter_input(&self, relative_path: &str, bytes: &[u8]) -> io::Result<()> {
        fs::write(self.root.join(relative_path), bytes)
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
            .env("LIQUIDFUN_TEST_REVISION", REVISION)
            .env("LIQUIDFUN_TEST_REMOTE_URL", REPOSITORY)
            .env("LIQUIDFUN_TEST_CMAKE_MARKER", &self.cmake_marker);
        Ok(command)
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
    let output_dir =
        workspace_root().join(format!("target/xtask-test-tools/{}", std::process::id()));
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

fn assert_failure_category(output: &Output, category: &str) {
    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert!(
        stderr(output).contains(category),
        "expected category `{category}` in stderr: {}",
        stderr(output)
    );
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
