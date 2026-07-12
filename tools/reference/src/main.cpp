#include "build_identity.hpp"
#include "collision_probe.hpp"
#include "math_probe.hpp"
#include "oracle_adapter.hpp"
#include "protocol.hpp"
#include "rigid_world.hpp"

#include <exception>
#include <cfenv>
#include <iostream>
#include <limits>
#include <stdexcept>
#include <string>

namespace {

std::string runtime_rounding_mode() {
  volatile float half_ulp_input =
      liquidfun::reference::float_from_bits(0x33800000U);
  volatile float one_input = 1.0F;
  volatile float odd_input =
      liquidfun::reference::float_from_bits(0x3F800001U);
  const auto half_ulp = half_ulp_input;
  const auto one = one_input;
  const auto odd = odd_input;
  const auto ties_even = (one + half_ulp) == one;
  const auto odd_rounds_up =
      liquidfun::reference::bits_from_float(odd + half_ulp) ==
      0x3F800002U;
  return std::fegetround() == FE_TONEAREST && ties_even && odd_rounds_up
             ? "nearest_ties_even"
             : "unsupported";
}

bool runtime_gradual_underflow() {
  volatile float minimum_normal_input = std::numeric_limits<float>::min();
  volatile float half_input = 0.5F;
  const auto half_minimum_normal = minimum_normal_input * half_input;
  return liquidfun::reference::bits_from_float(half_minimum_normal) ==
         0x00400000U;
}

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
      configured::kSanitizerMode,
      configured::kCompileCommandSha256,
      configured::kTargetTriple,
      configured::kTargetCpu,
      configured::kTargetFeatures,
      configured::kSdkOrSysroot,
      configured::kOptimization,
      configured::kFpModel,
      configured::kFpContract,
      configured::kDenormalMode,
      configured::kFeatureSet,
      configured::kOs,
      configured::kLibc,
      configured::kLibm,
      runtime_rounding_mode(),
      runtime_gradual_underflow()};
}

int run() {
  const auto identity = build_identity();
  const auto identity_sha256 =
      liquidfun::reference::build_identity_sha256(identity);
  liquidfun::reference::write_record(
      std::cout, liquidfun::reference::encode_handshake(identity));

  liquidfun::reference::OracleAdapter adapter;
  std::uint64_t math_probe_reset_epoch = 0;
  std::uint64_t collision_probe_reset_epoch = 0;
  liquidfun::reference::RigidWorldAdapter rigid_world_adapter;
  std::string line;
  while (liquidfun::reference::read_bounded_record(std::cin, line)) {
    const auto request_kind = liquidfun::reference::decode_request_kind(line);
    if (request_kind == liquidfun::reference::RequestKind::scenario) {
      const auto request = liquidfun::reference::decode_scenario_request(line);
      const auto trace = adapter.execute(request, identity_sha256);
      for (const auto& record : trace.records) {
        liquidfun::reference::write_record(std::cout, record);
      }
      continue;
    }
    if (request_kind == liquidfun::reference::RequestKind::collision_probe) {
      const auto batch = liquidfun::reference::execute_collision_probe(line);
      if (collision_probe_reset_epoch ==
          std::numeric_limits<std::uint64_t>::max()) {
        throw std::runtime_error("collision probe reset counter overflow");
      }
      for (const auto& result : batch.result_records) {
        liquidfun::reference::write_record(std::cout, result);
      }
      ++collision_probe_reset_epoch;
      liquidfun::reference::write_record(
          std::cout,
          liquidfun::reference::encode_collision_probe_end(
              batch, collision_probe_reset_epoch));
      continue;
    }
    if (request_kind == liquidfun::reference::RequestKind::rigid_world) {
      const auto trace = rigid_world_adapter.execute(line);
      liquidfun::reference::write_record(std::cout, trace.result_record);
      liquidfun::reference::write_record(std::cout, trace.end_record);
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
