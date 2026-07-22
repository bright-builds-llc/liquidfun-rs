#pragma once

#include "catalog_run_decode.hpp"

#include <string>
#include <vector>

namespace liquidfun::reference::catalog_run_detail {

std::vector<std::string> execute_payload(const CatalogRequest& request);

}  // namespace liquidfun::reference::catalog_run_detail
