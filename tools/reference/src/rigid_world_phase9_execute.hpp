#pragma once

#include "protocol.hpp"
#include "rigid_world_trace.hpp"

#include "nlohmann/json.hpp"

#include <Box2D/Box2D.h>

#include <algorithm>
#include <cstddef>
#include <cstdint>
#include <iterator>
#include <limits>
#include <memory>
#include <stdexcept>
#include <string>
#include <unordered_map>
#include <utility>
#include <vector>

namespace liquidfun::reference {
namespace phase9_detail {

using Json = nlohmann::json;

inline b2Vec2 phase9_vector(const Json& value) {
  return {
      float_from_bits(value.at("x_bits").get<std::uint32_t>()),
      float_from_bits(value.at("y_bits").get<std::uint32_t>())};
}

struct ParticleState {
  b2ParticleSystem* system = nullptr;
  const b2ParticleHandle* handle = nullptr;
};

struct SystemState {
  b2ParticleSystem* system = nullptr;
  std::size_t declared_capacity = 0;
  bool fixed = false;
  std::vector<uint32> flags;
  std::vector<b2Vec2> positions;
  std::vector<b2Vec2> velocities;
  std::vector<b2ParticleColor> colors;
  std::vector<void*> user_data;
};

class QueryCollector final : public b2QueryCallback {
 public:
  QueryCollector(
      const std::unordered_map<std::string, ParticleState>& particles,
      std::string control)
      : particles_(particles), control_(std::move(control)) {}

  bool ReportFixture(b2Fixture*) override { return true; }

  bool ReportParticle(const b2ParticleSystem* system, int32 index) override {
    const auto found = std::find_if(
        particles_.begin(), particles_.end(), [&](const auto& item) {
          return item.second.system == system && item.second.handle != nullptr &&
                 item.second.handle->GetIndex() == index;
        });
    if (found != particles_.end()) ids.push_back(found->first);
    if (control_ == "terminate") terminated = true;
    return !terminated;
  }

  std::vector<std::string> ids;
  bool terminated = false;

 private:
  const std::unordered_map<std::string, ParticleState>& particles_;
  std::string control_;
};

class RayCollector final : public b2RayCastCallback {
 public:
  RayCollector(
      const std::unordered_map<std::string, ParticleState>& particles,
      std::string control)
      : particles_(particles), control_(std::move(control)) {}

  float32 ReportFixture(
      b2Fixture*,
      const b2Vec2&,
      const b2Vec2&,
      float32) override {
    return 1.0F;
  }

  float32 ReportParticle(
      const b2ParticleSystem* system,
      int32 index,
      const b2Vec2&,
      const b2Vec2&,
      float32 fraction) override {
    const auto found = std::find_if(
        particles_.begin(), particles_.end(), [&](const auto& item) {
          return item.second.system == system && item.second.handle != nullptr &&
                 item.second.handle->GetIndex() == index;
        });
    if (found != particles_.end()) {
      ids.push_back(found->first);
      fractions.push_back(bits_from_float(fraction));
    }
    // LiquidFun takes min(current_fraction, callback_result), so a negative
    // Box2D-style ignore value would terminate particle traversal. Preserve
    // the protocol interval explicitly for both ignore and continue.
    if (control_ == "ignore") return 1.0F;
    if (control_ == "clip") return fraction;
    if (control_ == "terminate") {
      terminated = true;
      return 0.0F;
    }
    return 1.0F;
  }

  std::vector<std::string> ids;
  std::vector<std::uint32_t> fractions;
  bool terminated = false;

 private:
  const std::unordered_map<std::string, ParticleState>& particles_;
  std::string control_;
};

class TimelineExecution final
    : public b2ContactFilter,
      public b2ContactListener,
      public b2DestructionListener {
 public:
  TimelineExecution(b2World& world, Json timeline)
      : world_(world), timeline_(std::move(timeline)) {
    world_.SetContactFilter(this);
    world_.SetContactListener(this);
    world_.SetDestructionListener(this);
    for (const auto& body : timeline_.at("bodies")) {
      body_declarations_.emplace(body.at("body_id").get<std::string>(), body);
    }
    for (const auto& fixture : timeline_.at("fixtures")) {
      fixture_declarations_.emplace(
          fixture.at("fixture_id").get<std::string>(), fixture);
    }
    for (const auto& system : timeline_.at("particle_systems")) {
      system_declarations_.emplace(system.at("system_id").get<std::string>(), system);
    }
    for (const auto& particle : timeline_.at("particles")) {
      particle_declarations_.emplace(
          particle.at("particle_id").get<std::string>(), particle);
    }
  }

  ~TimelineExecution() override {
    world_.SetContactFilter(nullptr);
    world_.SetContactListener(nullptr);
    world_.SetDestructionListener(nullptr);
  }

  Json run() {
    Json checkpoints = Json::array();
    std::size_t next_checkpoint = 0;
    for (const auto& record : timeline_.at("actions")) {
      const auto& action = record.at("action");
      if (action.at("kind") == "particle") {
        execute(action.at("action"));
        phase9_affected_rigid_state_ = true;
      } else {
        execute_rigid(action);
      }
      while (next_checkpoint < timeline_.at("checkpoints").size() &&
             timeline_.at("checkpoints").at(next_checkpoint).at("after_action_id") ==
                 record.at("action_id")) {
        Json patch{
            {"checkpoint_id",
             timeline_.at("checkpoints").at(next_checkpoint).at("checkpoint_id")},
            {"phase", timeline_.at("checkpoints").at(next_checkpoint).at("phase")},
            {"checkpoint_index", next_checkpoint},
            {"counts", timeline_.at("checkpoints").at(next_checkpoint).at("counts")},
            {"observations", std::move(observations_)}};
        // Particle impulses can outlive their systems, so every downstream
        // checkpoint must come from this combined execution.
        if (phase9_affected_rigid_state_) {
          patch["bodies"] = body_snapshots();
          patch["fixtures"] = fixture_snapshots();
        }
        checkpoints.push_back(std::move(patch));
        observations_ = Json::array();
        ++next_checkpoint;
      }
    }
    for (auto& [id, state] : systems_) {
      static_cast<void>(id);
      world_.DestroyParticleSystem(state->system);
    }
    systems_.clear();
    particles_.clear();
    for (auto& [id, body] : bodies_) {
      static_cast<void>(id);
      world_.DestroyBody(body);
    }
    bodies_.clear();
    fixtures_.clear();
    return checkpoints;
  }

 private:
  bool ShouldCollide(b2Fixture*, b2Fixture*) override { return true; }

  bool ShouldCollide(b2Fixture*, b2ParticleSystem*, int32) override {
    return false;
  }

  bool ShouldCollide(b2ParticleSystem*, int32, int32) override {
    return false;
  }

  void BeginContact(
      b2ParticleSystem* system,
      b2ParticleContact* contact) override {
    record_occurrence(
        "contact_created",
        semantic_system_id(system),
        semantic_particle_id(system, contact->GetIndexA()),
        semantic_particle_id(system, contact->GetIndexB()));
  }

  void EndContact(
      b2ParticleSystem* system,
      int32 index_a,
      int32 index_b) override {
    record_occurrence(
        "contact_destroyed",
        semantic_system_id(system),
        semantic_particle_id(system, index_a),
        semantic_particle_id(system, index_b));
  }

  void BeginContact(
      b2ParticleSystem* system,
      b2ParticleBodyContact* contact) override {
    record_occurrence(
        "contact_created",
        semantic_system_id(system),
        semantic_particle_id(system, contact->index),
        nullptr,
        semantic_fixture_id(contact->fixture));
  }

  void EndContact(
      b2Fixture* fixture,
      b2ParticleSystem* system,
      int32 index) override {
    record_occurrence(
        "contact_destroyed",
        semantic_system_id(system),
        semantic_particle_id(system, index),
        nullptr,
        semantic_fixture_id(fixture));
  }

  void SayGoodbye(b2ParticleSystem* system, int32 index) override {
    record_occurrence(
        "particle_destroyed",
        semantic_system_id(system),
        semantic_particle_id(system, index));
  }

  void SayGoodbye(b2Joint*) override {}

  void SayGoodbye(b2Fixture*) override {}

#include "rigid_world_phase9_execute/state.hpp"

#include "rigid_world_phase9_execute/rigid.hpp"

#include "rigid_world_phase9_execute/particles.hpp"

#include "rigid_world_phase9_execute/observations.hpp"

  b2World& world_;
  Json timeline_;
  std::unordered_map<std::string, Json> body_declarations_;
  std::unordered_map<std::string, Json> fixture_declarations_;
  std::unordered_map<std::string, Json> system_declarations_;
  std::unordered_map<std::string, Json> particle_declarations_;
  std::unordered_map<std::string, std::unique_ptr<SystemState>> systems_;
  std::unordered_map<std::string, ParticleState> particles_;
  std::unordered_map<std::string, b2Vec2> particle_forces_;
  std::unordered_map<std::string, b2Body*> bodies_;
  std::unordered_map<std::string, b2Fixture*> fixtures_;
  Json observations_ = Json::array();
  Json occurrences_ = Json::array();
  std::uint32_t next_occurrence_ordinal_ = 0;
  bool phase9_affected_rigid_state_ = false;
};

}  // namespace phase9_detail

inline void apply_phase9_timeline(
    nlohmann::json& result,
    std::string_view raw_timeline) {
  b2World world({0.0F, 0.0F});
  auto patches = phase9_detail::TimelineExecution(
                     world,
                     nlohmann::json::parse(
                         raw_timeline.begin(), raw_timeline.end()))
                     .run();
  for (const auto& patch : patches) {
    const auto found = std::find_if(
        result.at("checkpoints").begin(),
        result.at("checkpoints").end(),
        [&](const auto& checkpoint) {
          return checkpoint.at("checkpoint_id") == patch.at("checkpoint_id");
        });
    if (found == result.at("checkpoints").end()) {
      nlohmann::json checkpoint{
          {"checkpoint_id", patch.at("checkpoint_id")},
          {"phase", patch.at("phase")},
          {"counts", patch.at("counts")},
          {"bodies", patch.contains("bodies") ? patch.at("bodies") : nlohmann::json::array()},
          {"fixtures", patch.contains("fixtures") ? patch.at("fixtures") : nlohmann::json::array()},
          {"contacts", nlohmann::json::array()},
          {"events", nlohmann::json::array()},
          {"destructions", nlohmann::json::array()},
          {"observations", patch.at("observations")}};
      const auto index = patch.at("checkpoint_index").get<std::size_t>();
      if (index > result.at("checkpoints").size()) {
        throw std::runtime_error("Phase 9 checkpoint insertion index is invalid");
      }
      result.at("checkpoints").insert(
          result.at("checkpoints").begin() + static_cast<std::ptrdiff_t>(index),
          std::move(checkpoint));
      continue;
    }
    (*found)["phase"] = patch.at("phase");
    if (patch.contains("bodies")) (*found)["bodies"] = patch.at("bodies");
    if (patch.contains("fixtures")) (*found)["fixtures"] = patch.at("fixtures");
    auto& observations = (*found)["observations"];
    if (observations.is_null()) observations = nlohmann::json::array();
    for (const auto& observation : patch.at("observations")) {
      observations.push_back(observation);
    }
  }
  if (world.GetParticleSystemList() != nullptr) {
    throw std::runtime_error("Phase 9 request left pinned particle state live");
  }
}

}  // namespace liquidfun::reference
