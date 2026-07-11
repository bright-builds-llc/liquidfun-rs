#include "build_identity.hpp"
#include "math_probe.hpp"
#include "oracle_adapter.hpp"
#include "protocol.hpp"

#include <exception>
#include <cfenv>
#include <iostream>
#include <limits>
#include <stdexcept>
#include <string>

namespace {

std::string runtime_rounding_mode() {
  const auto half_ulp = liquidfun::reference::float_from_bits(0x33800000U);
  const auto ties_even = (1.0F + half_ulp) == 1.0F;
  const auto odd_rounds_up =
      liquidfun::reference::bits_from_float(
          liquidfun::reference::float_from_bits(0x3F800001U) + half_ulp) ==
      0x3F800002U;
  return std::fegetround() == FE_TONEAREST && ties_even && odd_rounds_up
             ? "nearest_ties_even"
             : "unsupported";
}

bool runtime_gradual_underflow() {
  const auto half_minimum_normal =
      std::numeric_limits<float>::min() * 0.5F;
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
