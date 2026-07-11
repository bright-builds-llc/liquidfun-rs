#include "build_identity.hpp"
#include "math_probe.hpp"
#include "oracle_adapter.hpp"
#include "protocol.hpp"

#include <exception>
#include <iostream>
#include <limits>
#include <stdexcept>
#include <string>

namespace {

liquidfun::reference::BuildIdentity build_identity() {
  namespace configured = liquidfun::reference::configured_build_identity;
  return liquidfun::reference::BuildIdentity{
      configured::kOracleRevision,
      configured::kAdapterRevision,
      configured::kAdapterContentSha256,
      configured::kCmakePreset,
      configured::kCompilerId,
      configured::kCompilerVersion,
      configured::kTarget,
      configured::kBuildType,
      configured::kEffectiveCompileFlags,
      configured::kEffectiveLinkFlags,
      configured::kSanitizerMode};
}

int run() {
  const auto identity = build_identity();
  const auto identity_sha256 =
      liquidfun::reference::build_identity_sha256(identity);
  liquidfun::reference::write_record(
      std::cout, liquidfun::reference::encode_handshake(identity));

  liquidfun::reference::OracleAdapter adapter;
  std::uint64_t math_probe_reset_epoch = 0;
  std::string line;
  while (liquidfun::reference::read_bounded_record(std::cin, line)) {
    if (liquidfun::reference::decode_request_kind(line) ==
        liquidfun::reference::RequestKind::scenario) {
      const auto request = liquidfun::reference::decode_scenario_request(line);
      const auto trace = adapter.execute(request, identity_sha256);
      for (const auto& record : trace.records) {
        liquidfun::reference::write_record(std::cout, record);
      }
      continue;
    }
    const auto request = liquidfun::reference::decode_math_probe_request(line);
    const auto results = liquidfun::reference::execute_math_probe(request);
    if (results.size() > std::numeric_limits<std::uint32_t>::max() ||
        math_probe_reset_epoch == std::numeric_limits<std::uint64_t>::max()) {
      throw std::runtime_error("math probe result or reset counter overflow");
    }
    for (const auto& result : results) {
      liquidfun::reference::write_record(
          std::cout, liquidfun::reference::encode_math_probe_result(result));
    }
    ++math_probe_reset_epoch;
    liquidfun::reference::write_record(
        std::cout,
        liquidfun::reference::encode_math_probe_end(
            request, static_cast<std::uint32_t>(results.size()),
            math_probe_reset_epoch));
  }
  return 0;
}

}  // namespace

int main() {
  try {
    return run();
  } catch (const std::exception& error) {
    std::cerr << "liquidfun-reference: " << error.what() << '\n';
    return 1;
  }
}
