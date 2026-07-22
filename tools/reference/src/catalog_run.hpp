#pragma once

#include <cstdint>
#include <string>
#include <string_view>
#include <vector>

namespace liquidfun::reference {

struct CatalogRunTrace {
  std::vector<std::string> checkpoint_records;
  std::string end_record;
  std::uint64_t reset_epoch = 0;
};

class CatalogRunAdapter {
 public:
  CatalogRunTrace execute(
      std::string_view record,
      std::string_view actual_identity_sha256);

 private:
  std::uint64_t reset_epoch_ = 0;
};

}  // namespace liquidfun::reference
