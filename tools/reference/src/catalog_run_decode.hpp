#pragma once

#include "Box2D/Common/b2Math.h"
#include "nlohmann/json.hpp"

#include <cstdint>
#include <initializer_list>
#include <string>
#include <string_view>

namespace liquidfun::reference::catalog_run_detail {

using Json = nlohmann::json;

struct CatalogRequest {
  std::string request_id;
  std::string resolved_sha256;
  Json payload;
};

void require_members(
    const Json& value,
    std::initializer_list<std::string_view> expected,
    std::string_view context);
const Json& member(
    const Json& value,
    std::string_view name,
    std::string_view context);
std::uint32_t as_u32(const Json& value, std::string_view context);
std::string as_id(const Json& value, std::string_view context);
float as_finite_float(const Json& value, std::string_view context);
b2Vec2 as_vec2(const Json& value, std::string_view context);
CatalogRequest decode_request(
    std::string_view record,
    std::string_view actual_identity_sha256);

}  // namespace liquidfun::reference::catalog_run_detail
