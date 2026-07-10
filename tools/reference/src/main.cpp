#include "build_identity.hpp"
#include "oracle_adapter.hpp"
#include "protocol.hpp"

#include <exception>
#include <iostream>
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
  std::string line;
  while (liquidfun::reference::read_bounded_record(std::cin, line)) {
    const auto request = liquidfun::reference::decode_scenario_request(line);
    const auto trace = adapter.execute(request, identity_sha256);
    for (const auto& record : trace.records) {
      liquidfun::reference::write_record(std::cout, record);
    }
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
