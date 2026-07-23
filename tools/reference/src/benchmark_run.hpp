#pragma once

#include <cstdint>
#include <string>
#include <string_view>

namespace liquidfun::reference {

enum class BenchmarkRunEvent {
  authority_prepared,
  warmup_complete,
  measured_setup_complete,
  timer_started,
  timer_stopped,
  checkpoint_validated,
  teardown_complete
};

class BenchmarkRunObserver {
 public:
  virtual ~BenchmarkRunObserver() = default;
  virtual void observe(BenchmarkRunEvent event) = 0;
};

struct BenchmarkRunTrace {
  std::string result_record;
  std::uint64_t reset_epoch = 0;
};

class BenchmarkRunAdapter {
 public:
  explicit BenchmarkRunAdapter(BenchmarkRunObserver* maybe_observer = nullptr);

  BenchmarkRunTrace execute(std::string_view record);

 private:
  void observe(BenchmarkRunEvent event) const;

  BenchmarkRunObserver* maybe_observer_;
  std::uint64_t reset_epoch_ = 0;
};

}  // namespace liquidfun::reference
