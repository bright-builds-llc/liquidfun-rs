#pragma once

#include "catalog_run_decode.hpp"

#include <cstddef>
#include <memory>
#include <string>
#include <vector>

namespace liquidfun::reference::catalog_run_detail {

/// Request-local catalog world with setup and logical actions separated.
class CatalogExecutionSession {
 public:
  explicit CatalogExecutionSession(const CatalogRequest& request);
  ~CatalogExecutionSession();

  CatalogExecutionSession(CatalogExecutionSession&&) noexcept;
  CatalogExecutionSession& operator=(CatalogExecutionSession&&) noexcept;
  CatalogExecutionSession(const CatalogExecutionSession&) = delete;
  CatalogExecutionSession& operator=(const CatalogExecutionSession&) = delete;

  [[nodiscard]] std::size_t logical_action_count() const;
  [[nodiscard]] bool finished() const;
  void execute_next_logical_action();
  [[nodiscard]] std::string capture_current_checkpoint() const;

 private:
  class Impl;
  std::unique_ptr<Impl> impl_;
};

std::vector<std::string> execute_payload(const CatalogRequest& request);

}  // namespace liquidfun::reference::catalog_run_detail
