#pragma once

#include "catalog_run_decode.hpp"

#include "Box2D/Box2D.h"

#include <string_view>
#include <vector>

namespace liquidfun::reference::catalog_run_detail {

b2Joint* create_catalog_joint(
    std::string_view slug,
    const std::vector<b2Body*>& bodies,
    const std::vector<b2Joint*>& existing_joints,
    b2World& world);

void mutate_catalog_joint(b2Joint& joint, const Json& mutation);

}  // namespace liquidfun::reference::catalog_run_detail
