#include "oracle_adapter.hpp"
#include "protocol.hpp"

#include <exception>
#include <iostream>
#include <stdexcept>
#include <string>

namespace {

liquidfun::reference::BuildIdentity build_identity() {
  return liquidfun::reference::BuildIdentity{
      "7f20402173fd143a3988c921bc384459c6a858f2",
      "source-digest-v1",
      LIQUIDFUN_ADAPTER_CONTENT_SHA256,
      LIQUIDFUN_CMAKE_PRESET,
      LIQUIDFUN_COMPILER_ID,
      LIQUIDFUN_COMPILER_VERSION,
      LIQUIDFUN_TARGET,
      LIQUIDFUN_BUILD_TYPE,
      LIQUIDFUN_EFFECTIVE_COMPILE_FLAGS,
      LIQUIDFUN_EFFECTIVE_LINK_FLAGS,
      LIQUIDFUN_SANITIZER_MODE};
}

int run() {
  const auto identity = build_identity();
  const auto identity_sha256 =
      liquidfun::reference::build_identity_sha256(identity);
  liquidfun::reference::write_record(
      std::cout, liquidfun::reference::encode_handshake(identity));

  liquidfun::reference::OracleAdapter adapter;
  std::string line;
  while (std::getline(std::cin, line)) {
    if (!std::cin.eof()) {
      line.push_back('\n');
    }
    const auto request = liquidfun::reference::decode_scenario_request(line);
    const auto trace = adapter.execute(request, identity_sha256);
    for (const auto& record : trace.records) {
      liquidfun::reference::write_record(std::cout, record);
    }
  }
  if (!std::cin.eof()) {
    throw std::runtime_error("failed while reading protocol stdin");
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
