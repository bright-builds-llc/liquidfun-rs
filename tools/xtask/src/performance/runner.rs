//! Concrete identity-complete bridge to the Plan 12-20 paired runner.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_os = "linux")]
use std::fs;

use liquidfun_differential::{
    CatalogOracleSupervisor, NativeBenchmarkAdapter, OracleBenchmarkAdapter, OracleExecutable,
    OraclePreset, PairedBenchmarkOutcome, PairedBenchmarkPlan, PreparedNativeBenchmark,
    SessionProfile, run_paired_benchmark,
};
use liquidfun_test_protocol::{
    BuildIdentity,
    performance::{
        CompatibilityStatus, HardwareSession, PerformanceReportIdentity,
        PerformanceReportIdentityFields, ScalarOptimizationMode, benchmark_policy_sha256,
    },
};
use serde::Deserialize;
use serde_json::Value;

use super::paths::read_bounded_regular_file;
use super::{
    PairedCaseRequest, PairedRunFailure, PairedRunProvider, PerformanceCommandError,
    PerformanceEnvironment,
};

pub(super) struct ConcretePairedRunProvider {
    repository_root: PathBuf,
    oracle_revision: String,
    oracle_identity: BuildIdentity,
    rust_identity: RustIdentity,
    hardware: HardwareSession,
}

impl ConcretePairedRunProvider {
    pub(super) fn new(
        environment: &PerformanceEnvironment,
    ) -> Result<Self, PerformanceCommandError> {
        let root = environment.paths.repository_root();
        let oracle_revision = load_upstream_revision(&environment.paths.upstream_lock())?;
        let executable = OracleExecutable::resolve(root, OraclePreset::Release)
            .map_err(|error| PerformanceCommandError::new("oracle_release", error.to_string()))?;
        let mut supervisor = CatalogOracleSupervisor::new(
            executable,
            SessionProfile::Reuse,
            oracle_revision.clone(),
        );
        let oracle_identity = supervisor
            .discover_identity()
            .map_err(|error| PerformanceCommandError::new("oracle_identity", error.to_string()))?;
        Ok(Self {
            repository_root: root.to_path_buf(),
            oracle_revision,
            oracle_identity,
            rust_identity: collect_rust_identity(root)?,
            hardware: collect_hardware_session()?,
        })
    }
}

impl PairedRunProvider for ConcretePairedRunProvider {
    fn run_case(&mut self, request: PairedCaseRequest<'_>) -> Result<Value, PairedRunFailure> {
        let oracle_linker = format!("{}-linker", self.oracle_identity.compiler_id());
        let identity = PerformanceReportIdentity::new(PerformanceReportIdentityFields::new(
            request.scenario_id(),
            &self.rust_identity.revision,
            self.oracle_identity.oracle_revision(),
            &self.rust_identity.compiler,
            &self.rust_identity.linker,
            format!(
                "{} {}",
                self.oracle_identity.compiler_id(),
                self.oracle_identity.compiler_version()
            ),
            oracle_linker,
            self.oracle_identity.target(),
            &self.rust_identity.compile_flags,
            &self.rust_identity.link_flags,
            self.oracle_identity.effective_compile_flags(),
            self.oracle_identity.effective_link_flags(),
            self.hardware.clone(),
            benchmark_policy_sha256().map_err(|_| PairedRunFailure::Construction)?,
            request.matrix_sha256.clone(),
            request.case.catalog_sha256.clone(),
            request.resolved_sha256().clone(),
        ))
        .map_err(|_| PairedRunFailure::Construction)?;
        let prepared = PreparedNativeBenchmark::new(
            request.case.resolved.clone(),
            request.resolved_sha256(),
            request.case.logical_horizon,
            u32::from(request.policy().warmup_runs()),
        )
        .map_err(|_| PairedRunFailure::Construction)?;
        let mut native = NativeBenchmarkAdapter::new(prepared);
        let mut oracle = OracleBenchmarkAdapter::new(&self.repository_root, &self.oracle_revision)
            .map_err(|_| PairedRunFailure::Construction)?;
        let plan = PairedBenchmarkPlan::new(
            request.case.case_id.clone(),
            request.case.resolved.canonical_bytes().to_vec(),
            request.case.resolved.identity().settings(),
            request.workload(),
            request.size_point(),
            ScalarOptimizationMode::ReleaseScalar,
            request.case.logical_horizon,
            false,
            identity,
            CompatibilityStatus::D2Supported,
        )
        .map_err(|_| PairedRunFailure::Construction)?;
        match run_paired_benchmark(&plan, &mut native, &mut oracle) {
            PairedBenchmarkOutcome::Performance(report) => {
                serde_json::to_value(report).map_err(|_| PairedRunFailure::Construction)
            }
            PairedBenchmarkOutcome::PhysicsMismatch(_) => Err(PairedRunFailure::PhysicsMismatch),
            PairedBenchmarkOutcome::HarnessFailure(_) => Err(PairedRunFailure::Harness),
        }
    }
}

#[derive(Debug)]
struct RustIdentity {
    revision: String,
    compiler: String,
    linker: String,
    compile_flags: String,
    link_flags: String,
}

fn collect_rust_identity(root: &Path) -> Result<RustIdentity, PerformanceCommandError> {
    Ok(RustIdentity {
        revision: command_output(root, "git", &["rev-parse", "HEAD"])?,
        compiler: command_output(root, "rustc", &["-vV"])?,
        linker: "rustc-default-linker".to_owned(),
        compile_flags: std::env::var("RUSTFLAGS").unwrap_or_else(|_| "<default>".to_owned()),
        link_flags: "<rustc-default>".to_owned(),
    })
}

fn collect_hardware_session() -> Result<HardwareSession, PerformanceCommandError> {
    let logical_cores = std::thread::available_parallelism()
        .map_err(|error| PerformanceCommandError::new("hardware", error.to_string()))?
        .get();
    let logical_cores = u16::try_from(logical_cores)
        .map_err(|_| PerformanceCommandError::new("hardware", "logical core count overflow"))?;
    let hardware = collect_platform_hardware()?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| PerformanceCommandError::new("hardware", error.to_string()))?
        .as_secs();
    HardwareSession::new(
        format!("phase12-{now}-{}", std::process::id()),
        hardware.cpu_model,
        logical_cores,
        hardware.memory_bytes,
        hardware.operating_system,
    )
    .map_err(|error| PerformanceCommandError::new("hardware", error.to_string()))
}

struct PlatformHardware {
    cpu_model: String,
    memory_bytes: u64,
    operating_system: String,
}

#[cfg(target_os = "linux")]
fn collect_platform_hardware() -> Result<PlatformHardware, PerformanceCommandError> {
    let cpuinfo = fs::read_to_string("/proc/cpuinfo")
        .map_err(|error| PerformanceCommandError::new("hardware", error.to_string()))?;
    let meminfo = fs::read_to_string("/proc/meminfo")
        .map_err(|error| PerformanceCommandError::new("hardware", error.to_string()))?;
    let operating_system = command_output(Path::new("."), "uname", &["-srvmo"])?;
    parse_linux_hardware(&cpuinfo, &meminfo, &operating_system)
}

#[cfg(target_os = "macos")]
fn collect_platform_hardware() -> Result<PlatformHardware, PerformanceCommandError> {
    let cpu_model = command_output(
        Path::new("."),
        "sysctl",
        &["-n", "machdep.cpu.brand_string"],
    )
    .or_else(|_| command_output(Path::new("."), "sysctl", &["-n", "hw.model"]))?;
    let memory_bytes = command_output(Path::new("."), "sysctl", &["-n", "hw.memsize"])?
        .parse::<u64>()
        .map_err(|error| PerformanceCommandError::new("hardware", error.to_string()))?;
    let operating_system = command_output(Path::new("."), "sw_vers", &["-productVersion"])?;
    validate_hardware(cpu_model, memory_bytes, format!("macOS {operating_system}"))
}

#[cfg(target_os = "windows")]
fn collect_platform_hardware() -> Result<PlatformHardware, PerformanceCommandError> {
    let cpu_model = command_output(
        Path::new("."),
        "powershell",
        &[
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "(Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty Name)",
        ],
    )?;
    let memory_bytes = command_output(
        Path::new("."),
        "powershell",
        &[
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory",
        ],
    )?
    .parse::<u64>()
    .map_err(|error| PerformanceCommandError::new("hardware", error.to_string()))?;
    let operating_system = command_output(
        Path::new("."),
        "powershell",
        &[
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "(Get-CimInstance Win32_OperatingSystem).Caption",
        ],
    )?;
    validate_hardware(cpu_model, memory_bytes, operating_system)
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn collect_platform_hardware() -> Result<PlatformHardware, PerformanceCommandError> {
    Err(PerformanceCommandError::new(
        "hardware",
        "performance hardware collection is unsupported on this platform",
    ))
}

fn parse_linux_hardware(
    cpuinfo: &str,
    meminfo: &str,
    operating_system: &str,
) -> Result<PlatformHardware, PerformanceCommandError> {
    let cpu_model = cpuinfo
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once(':')?;
            matches!(key.trim(), "model name" | "Hardware").then(|| value.trim().to_owned())
        })
        .ok_or_else(|| PerformanceCommandError::new("hardware", "missing Linux CPU model"))?;
    let memory_kib = meminfo
        .lines()
        .find_map(|line| {
            let value = line.strip_prefix("MemTotal:")?.trim();
            value.split_ascii_whitespace().next()?.parse::<u64>().ok()
        })
        .ok_or_else(|| PerformanceCommandError::new("hardware", "missing Linux memory total"))?;
    let memory_bytes = memory_kib
        .checked_mul(1024)
        .ok_or_else(|| PerformanceCommandError::new("hardware", "Linux memory total overflow"))?;
    validate_hardware(cpu_model, memory_bytes, operating_system.to_owned())
}

fn validate_hardware(
    cpu_model: String,
    memory_bytes: u64,
    operating_system: String,
) -> Result<PlatformHardware, PerformanceCommandError> {
    let cpu_model = cpu_model.trim();
    let operating_system = operating_system.trim();
    if cpu_model.is_empty()
        || cpu_model == std::env::consts::ARCH
        || memory_bytes <= 1
        || operating_system.is_empty()
    {
        return Err(PerformanceCommandError::new(
            "hardware",
            "hardware identity contains a missing or placeholder value",
        ));
    }
    Ok(PlatformHardware {
        cpu_model: cpu_model.to_owned(),
        memory_bytes,
        operating_system: operating_system.to_owned(),
    })
}

fn command_output(
    current_dir: &Path,
    program: &str,
    args: &[&str],
) -> Result<String, PerformanceCommandError> {
    let output = Command::new(program)
        .args(args)
        .current_dir(current_dir)
        .output()
        .map_err(|error| PerformanceCommandError::new("identity", error.to_string()))?;
    if !output.status.success() {
        return Err(PerformanceCommandError::new(
            "identity",
            format!("{program} did not exit successfully"),
        ));
    }
    let value = String::from_utf8(output.stdout)
        .map_err(|error| PerformanceCommandError::new("identity", error.to_string()))?;
    let value = value.trim();
    if value.is_empty() {
        return Err(PerformanceCommandError::new(
            "identity",
            format!("{program} returned an empty identity"),
        ));
    }
    Ok(value.to_owned())
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UpstreamLock {
    schema_version: u8,
    repository: String,
    revision: String,
    release_tag: String,
    release_tag_object: String,
    release_commit: String,
    submodule_path: String,
    patch_set: String,
}

fn load_upstream_revision(path: &Path) -> Result<String, PerformanceCommandError> {
    let bytes = read_bounded_regular_file(path)
        .map_err(|message| PerformanceCommandError::new("upstream_lock", message))?;
    let lock: UpstreamLock = toml::from_slice(&bytes)
        .map_err(|error| PerformanceCommandError::new("upstream_lock", error.to_string()))?;
    if lock.schema_version != 1
        || lock.repository != "https://github.com/google/liquidfun.git"
        || lock.revision.len() != 40
        || !lock.revision.bytes().all(|byte| byte.is_ascii_hexdigit())
        || lock.release_tag.is_empty()
        || lock.release_tag_object.is_empty()
        || lock.release_commit.is_empty()
        || lock.submodule_path != "third_party/liquidfun"
        || lock.patch_set != "none"
    {
        return Err(PerformanceCommandError::new(
            "upstream_lock",
            "upstream lock is not the reviewed exact revision",
        ));
    }
    Ok(lock.revision)
}

#[cfg(test)]
mod tests {
    use super::parse_linux_hardware;

    #[test]
    fn linux_hardware_parser_accepts_concrete_cpu_and_memory() {
        // Arrange
        let cpuinfo = "processor : 0\nmodel name : Reviewed CPU 9000\n";
        let meminfo = "MemTotal:       16384 kB\n";

        // Act
        let hardware = parse_linux_hardware(cpuinfo, meminfo, "Linux 6.12 x86_64")
            .expect("concrete Linux hardware should validate");

        // Assert
        assert_eq!(hardware.cpu_model, "Reviewed CPU 9000");
        assert_eq!(hardware.memory_bytes, 16_777_216);
    }

    #[test]
    fn linux_hardware_parser_rejects_architecture_only_cpu_model() {
        // Arrange
        let cpuinfo = format!("model name : {}\n", std::env::consts::ARCH);
        let meminfo = "MemTotal:       16384 kB\n";

        // Act
        let result = parse_linux_hardware(&cpuinfo, meminfo, "Linux 6.12 x86_64");

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn linux_hardware_parser_rejects_one_byte_memory_fallback() {
        // Arrange
        let cpuinfo = "model name : Reviewed CPU 9000\n";
        let meminfo = "MemTotal:       0 kB\n";

        // Act
        let result = parse_linux_hardware(cpuinfo, meminfo, "Linux 6.12 x86_64");

        // Assert
        assert!(result.is_err());
    }
}
