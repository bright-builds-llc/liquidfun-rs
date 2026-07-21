#pragma once

#include "protocol.hpp"
#include "rigid_world_trace.hpp"

#include "nlohmann/json.hpp"

#include <Box2D/Box2D.h>

#include <algorithm>
#include <cstddef>
#include <cstdint>
#include <limits>
#include <memory>
#include <iterator>
#include <stdexcept>
#include <string>
#include <unordered_map>
#include <utility>
#include <vector>

namespace liquidfun::reference {
namespace phase10_detail {

using Json = nlohmann::json;

inline b2Vec2 phase10_vector(const Json& value) {
  return {
      float_from_bits(value.at("x_bits").get<std::uint32_t>()),
      float_from_bits(value.at("y_bits").get<std::uint32_t>())};
}

struct ParticleBinding {
  std::string id;
  std::string system_id;
  b2ParticleSystem* system = nullptr;
  const b2ParticleHandle* handle = nullptr;
};

struct GroupBinding {
  std::string id;
  std::string system_id;
  b2ParticleGroup* group = nullptr;
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

class TimelineExecution final
    : public b2ContactListener,
      public b2DestructionListener {
 public:
  TimelineExecution(b2World& world, Json timeline)
      : world_(world), timeline_(std::move(timeline)) {
    world_.SetContactListener(this);
    world_.SetDestructionListener(this);
    for (const auto& declaration : timeline_.at("particle_systems")) {
      system_declarations_.emplace(
          declaration.at("system_id").get<std::string>(), declaration);
    }
    for (const auto& declaration : timeline_.at("bodies")) {
      body_declarations_.emplace(
          declaration.at("body_id").get<std::string>(), declaration);
    }
    for (const auto& declaration : timeline_.at("fixtures")) {
      fixture_declarations_.emplace(
          declaration.at("fixture_id").get<std::string>(), declaration);
    }
  }

  ~TimelineExecution() override {
    world_.SetContactListener(nullptr);
    world_.SetDestructionListener(nullptr);
  }

  Json run() {
    Json patches = Json::array();
    std::size_t next_checkpoint = 0;
    for (const auto& record : timeline_.at("actions")) {
      execute(record.at("action"));
      while (next_checkpoint < timeline_.at("checkpoints").size() &&
             timeline_.at("checkpoints").at(next_checkpoint).at("after_action_id") ==
                 record.at("action_id")) {
        const auto& checkpoint = timeline_.at("checkpoints").at(next_checkpoint);
        patches.push_back(
            {{"checkpoint_id", checkpoint.at("checkpoint_id")},
             {"phase", checkpoint.at("phase")},
             {"checkpoint_index", next_checkpoint},
             {"counts", checkpoint.at("counts")},
             {"observations", std::move(observations_)}});
        observations_ = Json::array();
        ++next_checkpoint;
      }
    }
    cleanup();
    return patches;
  }

 private:
  void SayGoodbye(b2Joint*) override {}
  void SayGoodbye(b2Fixture*) override {}

  void SayGoodbye(b2ParticleGroup* group) override {
    const auto found = std::find_if(
        groups_.begin(), groups_.end(),
        [&](const auto& binding) { return binding.group == group; });
    if (found == groups_.end()) return;
    if (!suppress_group_destroy_event_) {
      add_event("group_destroyed", found->system_id, found->id, nullptr, nullptr, nullptr);
    }
    found->group = nullptr;
  }

  void SayGoodbye(b2ParticleSystem* system, int32 index) override {
    const auto* binding = particle_binding(system, index);
    if (binding == nullptr) return;
    add_event(
        "particle_destroyed", binding->system_id, nullptr, binding->id,
        nullptr, nullptr);
  }

  void BeginContact(
      b2ParticleSystem* system,
      b2ParticleContact* contact) override {
    const auto* a = particle_binding(system, contact->GetIndexA());
    const auto* b = particle_binding(system, contact->GetIndexB());
    if (a != nullptr && b != nullptr) {
      add_event(
          "particle_contact_begin", a->system_id, nullptr, a->id, b->id,
          nullptr);
    }
  }

  void EndContact(
      b2ParticleSystem* system,
      int32 index_a,
      int32 index_b) override {
    const auto* a = particle_binding(system, index_a);
    const auto* b = particle_binding(system, index_b);
    if (a != nullptr && b != nullptr) {
      add_event(
          "particle_contact_end", a->system_id, nullptr, a->id, b->id,
          nullptr);
    }
  }

  void BeginContact(
      b2ParticleSystem* system,
      b2ParticleBodyContact* contact) override {
    const auto* particle = particle_binding(system, contact->index);
    if (particle == nullptr) return;
    const auto body_id = semantic_body_id(contact->body);
    add_event(
        "body_contact_begin", particle->system_id, nullptr, particle->id,
        nullptr, body_id);
  }

  void EndContact(
      b2Fixture* fixture,
      b2ParticleSystem* system,
      int32 index) override {
    const auto* particle = particle_binding(system, index);
    if (particle == nullptr) return;
    add_event(
        "body_contact_end", particle->system_id, nullptr, particle->id,
        nullptr, semantic_body_id(fixture->GetBody()));
  }

  SystemState& system(const Json& raw_id) {
    const auto found = systems_.find(raw_id.get<std::string>());
    if (found == systems_.end()) {
      throw std::runtime_error("Phase 10 particle system is not live");
    }
    return *found->second;
  }

  GroupBinding& group(const Json& raw_id) {
    const auto id = raw_id.get<std::string>();
    const auto found = std::find_if(
        groups_.begin(), groups_.end(),
        [&](const auto& binding) { return binding.id == id; });
    if (found == groups_.end() || found->group == nullptr) {
      throw std::runtime_error("Phase 10 particle group is not live");
    }
    return *found;
  }

  const GroupBinding& group(const Json& raw_id) const {
    const auto id = raw_id.get<std::string>();
    const auto found = std::find_if(
        groups_.begin(), groups_.end(),
        [&](const auto& binding) { return binding.id == id; });
    if (found == groups_.end() || found->group == nullptr) {
      throw std::runtime_error("Phase 10 particle group is not live");
    }
    return *found;
  }

  const ParticleBinding* particle_binding(
      const b2ParticleSystem* system,
      int32 index) const {
    const auto found = std::find_if(
        particles_.begin(), particles_.end(), [&](const auto& binding) {
          return binding.system == system && binding.handle != nullptr &&
                 binding.handle->GetIndex() == index;
        });
    return found == particles_.end() ? nullptr : &*found;
  }

  std::string semantic_particle_id(
      const b2ParticleSystem* system,
      int32 index) const {
    const auto* binding = particle_binding(system, index);
    if (binding == nullptr) {
      throw std::runtime_error(
          "Phase 10 particle has no semantic identity at dense index " +
          std::to_string(index) + " of " +
          std::to_string(system->GetParticleCount()) + " with flags " +
          std::to_string(system->GetFlagsBuffer()[index]));
    }
    return binding->id;
  }

  std::string semantic_body_id(const b2Body* body) const {
    const auto found = std::find_if(
        bodies_.begin(), bodies_.end(),
        [&](const auto& item) { return item.second == body; });
    return found == bodies_.end() ? std::string{} : found->first;
  }

  std::string semantic_fixture_id(const b2Fixture* fixture) const {
    const auto found = std::find_if(
        fixtures_.begin(), fixtures_.end(),
        [&](const auto& item) { return item.second == fixture; });
    if (found == fixtures_.end()) {
      throw std::runtime_error("Phase 10 body contact has no fixture identity");
    }
    return found->first;
  }

#include "rigid_world_phase10_operations.hpp"
#include "rigid_world_phase10_capture.hpp"
  void refresh_particle_handles() {
    for (auto& binding : particles_) {
      binding.handle = nullptr;
      const auto count = binding.system->GetParticleCount();
      for (int32 index = 0; index < count; ++index) {
        const auto* raw_token = static_cast<const std::string*>(
            binding.system->GetUserDataBuffer()[index]);
        if (raw_token != nullptr && *raw_token == binding.id &&
            (binding.system->GetFlagsBuffer()[index] & b2_zombieParticle) == 0U) {
          binding.handle = binding.system->GetParticleHandleFromIndex(index);
          break;
        }
      }
    }
  }

  void discard_dead() {
    particles_.erase(
        std::remove_if(
            particles_.begin(), particles_.end(), [](const auto& binding) {
              return binding.handle == nullptr ||
                     binding.handle->GetIndex() == b2_invalidParticleIndex;
            }),
        particles_.end());
    groups_.erase(
        std::remove_if(
            groups_.begin(), groups_.end(),
            [](const auto& binding) { return binding.group == nullptr; }),
        groups_.end());
  }

  void cleanup() {
    for (auto& [id, state] : systems_) {
      static_cast<void>(id);
      world_.DestroyParticleSystem(state->system);
    }
    systems_.clear();
    particles_.clear();
    groups_.clear();
    for (auto& [id, body] : bodies_) {
      static_cast<void>(id);
      world_.DestroyBody(body);
    }
    bodies_.clear();
    fixtures_.clear();
  }

  b2World& world_;
  Json timeline_;
  std::unordered_map<std::string, Json> system_declarations_;
  std::unordered_map<std::string, Json> body_declarations_;
  std::unordered_map<std::string, Json> fixture_declarations_;
  std::unordered_map<std::string, std::unique_ptr<SystemState>> systems_;
  std::vector<ParticleBinding> particles_;
  std::vector<std::unique_ptr<std::string>> particle_tokens_;
  std::vector<GroupBinding> groups_;
  std::unordered_map<std::string, b2Vec2> velocity_before_;
  std::unordered_map<std::string, b2Body*> bodies_;
  std::unordered_map<std::string, b2Fixture*> fixtures_;
  Json provenance_;
  Json observations_ = Json::array();
  Json events_ = Json::array();
  Json witnesses_ = Json::array();
  std::uint32_t next_phase9_occurrence_ordinal_ = 0;
  bool suppress_group_destroy_event_ = false;
};

}  // namespace phase10_detail

inline void apply_phase10_timeline(
    nlohmann::json& result,
    std::string_view raw_timeline) {
  b2World world({0.0F, 0.0F});
  auto patches = phase10_detail::TimelineExecution(
                     world,
                     nlohmann::json::parse(
                         raw_timeline.begin(), raw_timeline.end()))
                     .run();
  for (const auto& patch : patches) {
    const auto found = std::find_if(
        result.at("checkpoints").begin(), result.at("checkpoints").end(),
        [&](const auto& checkpoint) {
          return checkpoint.at("checkpoint_id") == patch.at("checkpoint_id");
        });
    if (found == result.at("checkpoints").end()) {
      nlohmann::json checkpoint{
          {"checkpoint_id", patch.at("checkpoint_id")},
          {"phase", patch.at("phase")},
          {"counts", patch.at("counts")},
          {"bodies", nlohmann::json::array()},
          {"fixtures", nlohmann::json::array()},
          {"contacts", nlohmann::json::array()},
          {"events", nlohmann::json::array()},
          {"destructions", nlohmann::json::array()},
          {"observations", patch.at("observations")}};
      const auto index = patch.at("checkpoint_index").get<std::size_t>();
      result.at("checkpoints").insert(
          result.at("checkpoints").begin() + static_cast<std::ptrdiff_t>(index),
          std::move(checkpoint));
      continue;
    }
    (*found)["phase"] = patch.at("phase");
    auto& observations = (*found)["observations"];
    if (observations.is_null()) observations = nlohmann::json::array();
    for (const auto& observation : patch.at("observations")) {
      observations.push_back(observation);
    }
  }
  if (world.GetParticleSystemList() != nullptr) {
    throw std::runtime_error("Phase 10 request left pinned particle state live");
  }
}

}  // namespace liquidfun::reference
