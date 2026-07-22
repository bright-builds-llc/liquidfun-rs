#include "catalog_run.hpp"

#include "catalog_checkpoint.hpp"
#include "catalog_run_decode.hpp"
#include "catalog_run_session.hpp"
#include "protocol.hpp"

#include <limits>
#include <stdexcept>
#include <utility>

namespace liquidfun::reference {

CatalogRunTrace CatalogRunAdapter::execute(
    std::string_view record,
    std::string_view actual_identity_sha256) {
  const auto request = catalog_run_detail::decode_request(
      record, actual_identity_sha256);
  auto records = catalog_run_detail::execute_payload(request);
  std::size_t trace_bytes = 0;
  for (const auto& checkpoint : records) {
    trace_bytes += checkpoint.size();
    if (trace_bytes > kMaximumTraceBytes) {
      throw std::runtime_error("catalog trace exceeds output limit");
    }
  }
  if (reset_epoch_ == std::numeric_limits<std::uint64_t>::max()) {
    throw std::runtime_error("catalog reset epoch overflow");
  }
  ++reset_epoch_;
  const auto checkpoint_count = static_cast<std::uint32_t>(records.size());
  return {std::move(records),
          encode_catalog_run_end(request.request_id, request.resolved_sha256,
                                 checkpoint_count, reset_epoch_),
          reset_epoch_};
}

}  // namespace liquidfun::reference
