// Repository-authored semantic probe for pinned LiquidFun behavior.
// No upstream source or Rust-produced expectation is copied into this file.

#include "build_identity.hpp"
#include "phase10_group_topology_cases.hpp"
#include "protocol.hpp"

#include <nlohmann/json.hpp>

#include <chrono>
#include <ctime>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <vector>

namespace {

using Json = nlohmann::json;

constexpr std::string_view kExpectedOracleRevision =
    "7f20402173fd143a3988c921bc384459c6a858f2";
constexpr std::string_view kCmakeTarget = "phase10-group-topology-witness";

struct CommandLine {
  std::string output_path;
  std::string provenance_path;
  std::vector<std::string> exact_argv;
};

CommandLine parse_command_line(int argc, char** argv) {
  if (argc != 5 || std::string_view(argv[1]) != "--output" ||
      std::string_view(argv[3]) != "--provenance") {
    throw std::runtime_error(
        "usage: phase10-group-topology-witness --output <path> "
        "--provenance <path>");
  }
  if (std::string_view(argv[2]).empty() || std::string_view(argv[4]).empty() ||
      std::string_view(argv[2]) == std::string_view(argv[4])) {
    throw std::runtime_error("output and provenance paths must be distinct");
  }

  CommandLine command_line;
  command_line.output_path = argv[2];
  command_line.provenance_path = argv[4];
  command_line.exact_argv.reserve(static_cast<std::size_t>(argc));
  for (int index = 0; index < argc; ++index) {
    command_line.exact_argv.emplace_back(argv[index]);
  }
  return command_line;
}

std::string utc_timestamp() {
  const std::time_t now = std::chrono::system_clock::to_time_t(
      std::chrono::system_clock::now());
  std::tm utc{};
#if defined(_WIN32)
  if (gmtime_s(&utc, &now) != 0) {
#else
  if (gmtime_r(&now, &utc) == nullptr) {
#endif
    throw std::runtime_error("failed to generate UTC timestamp");
  }
  std::ostringstream output;
  output << std::put_time(&utc, "%Y-%m-%dT%H:%M:%SZ");
  return output.str();
}

void write_json(const std::string& path, const Json& document) {
  std::ofstream output(path, std::ios::binary | std::ios::trunc);
  if (!output) {
    throw std::runtime_error("failed to open output path: " + path);
  }
  output << document.dump(2) << '\n';
  output.flush();
  if (!output) {
    throw std::runtime_error("failed to write output path: " + path);
  }
}

}  // namespace

int main(int argc, char** argv) {
  try {
    const CommandLine command_line = parse_command_line(argc, argv);
    namespace identity = liquidfun::reference::configured_build_identity;
    if (std::string_view(identity::kOracleRevision) != kExpectedOracleRevision) {
      throw std::runtime_error("configured upstream revision is not pinned");
    }

    const Json witnesses = Json{
        {"schema_version", 1},
        {"oracle_revision", identity::kOracleRevision},
        {"cases", capture_phase10_group_topology_cases()},
    };
    const std::string witness_bytes = witnesses.dump(2) + '\n';
    const std::string witness_sha256 =
        liquidfun::reference::sha256_hex(witness_bytes);
    write_json(command_line.output_path, witnesses);

    const Json provenance = Json{
        {"schema_version", 1},
        {"oracle_revision", identity::kOracleRevision},
        {"adapter_content_sha256", identity::kAdapterContentSha256},
        {"probe_source_sha256", PHASE10_GROUP_TOPOLOGY_PROBE_SOURCE_SHA256},
        {"compiler_id", identity::kCompilerId},
        {"compiler_version", identity::kCompilerVersion},
        {"target", identity::kTarget},
        {"cmake_preset", identity::kCmakePreset},
        {"cmake_target", kCmakeTarget},
        {"exact_argv", command_line.exact_argv},
        {"generation_timestamp", utc_timestamp()},
        {"witness_sha256", witness_sha256},
    };
    write_json(command_line.provenance_path, provenance);
    std::cout << "phase10 group/topology witnesses: " << witness_sha256 << '\n';
    return 0;
  } catch (const std::exception& error) {
    std::cerr << "phase10 group/topology witness error: " << error.what()
              << '\n';
    return 1;
  }
}
