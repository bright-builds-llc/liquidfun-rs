#pragma once

#include <cstdint>
#include <string>
#include <string_view>
#include <vector>

namespace liquidfun::reference {

struct CollisionProbeBatch {
  std::string request_id;
  std::vector<std::string> result_records;
};

CollisionProbeBatch execute_collision_probe(std::string_view record);
std::string encode_collision_probe_end(
    const CollisionProbeBatch& batch,
    std::uint64_t reset_epoch);

}  // namespace liquidfun::reference
