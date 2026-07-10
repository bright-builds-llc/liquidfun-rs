#pragma once

#include "protocol.hpp"

#include <cstdint>
#include <string>
#include <vector>

namespace liquidfun::reference {

struct OracleTrace {
  std::vector<std::string> records;
  std::uint64_t reset_epoch = 0;
  bool reset_verified = false;
};

class OracleAdapter {
 public:
  OracleTrace execute(
      const ScenarioRequest& request,
      const std::string& identity_sha256);

 private:
  std::uint64_t reset_epoch_ = 0;
};

}  // namespace liquidfun::reference
