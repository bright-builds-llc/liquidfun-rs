#pragma once

#include "protocol.hpp"
#include "nlohmann/json.hpp"

#include <cstdint>
#include <string>
#include <string_view>

namespace liquidfun::reference {

struct CatalogCheckpointInput {
  std::string request_id;
  std::string resolved_sha256;
  std::string checkpoint_id;
  std::uint32_t logical_step = 0;
  std::uint32_t simulation_time_bits = 0;
  WorldCounts counts;
  std::uint32_t debug_primitive_count = 0;
  nlohmann::json debug_primitives = nlohmann::json::array();
};

std::string encode_catalog_checkpoint(const CatalogCheckpointInput& input);
std::string encode_catalog_run_end(
    std::string_view request_id,
    std::string_view resolved_sha256,
    std::uint32_t checkpoint_count,
    std::uint64_t reset_epoch);

}  // namespace liquidfun::reference
