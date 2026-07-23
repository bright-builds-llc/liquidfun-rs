#pragma once

#include "math_probe.hpp"

#include <cstddef>
#include <cstdint>
#include <iosfwd>
#include <string>
#include <string_view>
#include <vector>

namespace liquidfun::reference {

inline constexpr std::uint32_t kProtocolVersion = 1;
inline constexpr std::uint32_t kScenarioSchemaVersion = 1;
inline constexpr std::uint32_t kTraceSchemaVersion = 1;
inline constexpr std::uint32_t kToleranceProfileVersion = 1;
inline constexpr std::size_t kMaximumRecordBytes = 1024U * 1024U;
inline constexpr std::size_t kMaximumTraceBytes = 32U * 1024U * 1024U;
inline constexpr std::size_t kMaximumDepth = 32;
inline constexpr std::size_t kMaximumStringBytes = 4096;
inline constexpr std::size_t kMaximumCollectionItems = 4096;
inline constexpr std::size_t kMaximumObservableItems = 128;
inline constexpr std::size_t kMaximumIdBytes = 128;

enum class ScenarioSourceKind { named, seeded };

enum class RequestKind {
  scenario,
  math_probe,
  collision_probe,
  rigid_world,
  catalog_run,
  benchmark_run
};

struct ScenarioSource {
  ScenarioSourceKind kind = ScenarioSourceKind::named;
  std::string name;
  std::string generator_id;
  std::uint32_t generator_version = 0;
  std::uint64_t seed = 0;
};

struct StepCommand {
  std::string command_id;
  std::uint32_t timestep_bits = 0;
  std::uint32_t velocity_iterations = 0;
  std::uint32_t position_iterations = 0;
  std::uint32_t particle_iterations = 0;
};

enum class Observable { world_counts, simulation_time };

struct CheckpointRequest {
  std::string checkpoint_id;
  std::string after_command_id;
  std::string phase;
  std::vector<Observable> observables;
};

struct ScenarioV1 {
  std::string scenario_id;
  ScenarioSource source;
  std::uint32_t gravity_x_bits = 0;
  std::uint32_t gravity_y_bits = 0;
  std::vector<StepCommand> commands;
  std::vector<CheckpointRequest> checkpoints;
};

struct ScenarioRequest {
  std::string request_id;
  std::string tolerance_profile_sha256;
  ScenarioV1 scenario;
};

struct BuildIdentity {
  std::string oracle_revision;
  std::string adapter_revision;
  std::string adapter_content_sha256;
  std::string cmake_preset;
  std::string compiler_id;
  std::string compiler_version;
  std::string target;
  std::string build_type;
  std::string effective_compile_flags;
  std::string effective_link_flags;
  std::string sanitizer_mode;
  std::string compile_command_sha256;
  std::string target_triple;
  std::string target_cpu;
  std::string target_features;
  std::string sdk_or_sysroot;
  std::string optimization;
  std::string fp_model;
  std::string fp_contract;
  std::string denormal_mode;
  std::string feature_set;
  std::string os;
  std::string libc;
  std::string libm;
  std::string rounding_mode;
  bool gradual_underflow = false;
};

struct WorldCounts {
  std::uint32_t bodies = 0;
  std::uint32_t fixtures = 0;
  std::uint32_t joints = 0;
  std::uint32_t contacts = 0;
  std::uint32_t particle_systems = 0;
  std::uint32_t particle_groups = 0;
  std::uint32_t particles = 0;
};

struct BenchmarkRunSettings {
  std::uint32_t timestep_bits = 0;
  std::uint32_t velocity_iterations = 0;
  std::uint32_t position_iterations = 0;
  std::uint32_t particle_iterations = 0;
};

struct BenchmarkRunIdentity {
  std::string request_id;
  std::string resolved_sha256;
  BenchmarkRunSettings settings;
  std::string workload;
  std::string size_point;
  std::string optimization_mode;
  std::uint32_t warmup_count = 0;
  std::uint32_t measured_horizon = 0;
  std::uint32_t sample_ordinal = 0;
  std::string policy_sha256;
  bool profile_enabled = false;
};

struct BenchmarkRunRequest {
  BenchmarkRunIdentity identity;
  std::string resolved_bytes;
};

ScenarioRequest decode_scenario_request(std::string_view record);
RequestKind decode_request_kind(std::string_view record);
MathProbeRequest decode_math_probe_request(std::string_view record);
BenchmarkRunRequest decode_benchmark_run_request(std::string_view record);
std::string encode_scenario_request(const ScenarioRequest& request);
std::string encode_scenario(const ScenarioV1& scenario);
std::string encode_handshake(const BuildIdentity& identity);
std::string encode_trace_begin(
    const ScenarioRequest& request,
    std::string_view scenario_sha256,
    std::string_view identity_sha256);
std::string encode_checkpoint(
    const ScenarioRequest& request,
    const CheckpointRequest& checkpoint,
    std::uint32_t ordinal,
    std::uint32_t simulation_time_bits,
    const WorldCounts& counts,
    std::string_view identity_sha256);
std::string encode_trace_end(
    const ScenarioRequest& request,
    std::uint32_t checkpoint_count,
    std::string_view trace_payload_sha256,
    std::uint64_t reset_epoch,
    bool reset_verified,
    std::string_view identity_sha256);
std::string encode_math_probe_result(const MathProbeResult& result);
std::string encode_math_probe_end(
    const MathProbeRequest& request,
    std::uint32_t result_count,
    std::uint64_t reset_epoch);
std::string trace_payload_sha256(
    const std::vector<std::string>& checkpoint_records);
std::string build_identity_sha256(const BuildIdentity& identity);
std::string sha256_hex(std::string_view bytes);
float float_from_bits(std::uint32_t bits);
std::uint32_t bits_from_float(float value);
bool read_bounded_record(std::istream& input, std::string& record);
void validate_bounded_json_record(std::string_view record);
void write_record(std::ostream& output, std::string_view record);

}  // namespace liquidfun::reference
