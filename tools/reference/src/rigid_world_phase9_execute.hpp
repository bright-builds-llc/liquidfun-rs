#pragma once

#include "protocol.hpp"
#include "rigid_world_trace.hpp"

#include "nlohmann/json.hpp"

#include <Box2D/Box2D.h>

#include <algorithm>
#include <cstdint>
#include <iterator>
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
  explicit QueryCollector(const std::unordered_map<std::string, ParticleState>& particles)
      : particles_(particles) {}

  bool ReportFixture(b2Fixture*) override { return true; }

  bool ReportParticle(const b2ParticleSystem* system, int32 index) override {
    const auto found = std::find_if(
        particles_.begin(), particles_.end(), [&](const auto& item) {
          return item.second.system == system && item.second.handle != nullptr &&
                 item.second.handle->GetIndex() == index;
        });
    if (found != particles_.end()) ids.push_back(found->first);
    return true;
  }

  std::vector<std::string> ids;

 private:
  const std::unordered_map<std::string, ParticleState>& particles_;
};

class RayCollector final : public b2RayCastCallback {
 public:
  explicit RayCollector(const std::unordered_map<std::string, ParticleState>& particles)
      : particles_(particles) {}

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
    return 1.0F;
  }

  std::vector<std::string> ids;
  std::vector<std::uint32_t> fractions;

 private:
  const std::unordered_map<std::string, ParticleState>& particles_;
};

class TimelineExecution {
 public:
  TimelineExecution(b2World& world, Json timeline)
      : world_(world), timeline_(std::move(timeline)) {
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

  Json run() {
    Json checkpoints = Json::array();
    std::size_t next_checkpoint = 0;
    for (const auto& record : timeline_.at("actions")) {
      const auto& action = record.at("action");
      if (action.at("kind") == "particle") {
        execute(action.at("action"));
        phase9_since_checkpoint_ = true;
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
            {"observations", std::move(observations_)}};
        if (phase9_since_checkpoint_) patch["bodies"] = body_snapshots();
        checkpoints.push_back(std::move(patch));
        observations_ = Json::array();
        phase9_since_checkpoint_ = false;
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
  SystemState& system(const Json& raw_id) {
    const auto found = systems_.find(raw_id.get<std::string>());
    if (found == systems_.end()) {
      throw std::runtime_error("Phase 9 particle system is not live");
    }
    return *found->second;
  }

  ParticleState& particle(const Json& raw_id) {
    const auto found = particles_.find(raw_id.get<std::string>());
    if (found == particles_.end() || found->second.handle == nullptr ||
        found->second.handle->GetIndex() == b2_invalidParticleIndex) {
      throw std::runtime_error("Phase 9 particle is not live");
    }
    return found->second;
  }

  Json semantic_particle_ids() const {
    Json ids = Json::array();
    for (const auto& declaration : timeline_.at("particles")) {
      const auto id = declaration.at("particle_id").get<std::string>();
      const auto found = particles_.find(id);
      if (found != particles_.end() && found->second.handle != nullptr &&
          found->second.handle->GetIndex() != b2_invalidParticleIndex) {
        ids.push_back(id);
      }
    }
    return ids;
  }

  static b2BodyType body_type(std::string_view kind) {
    if (kind == "static") return b2_staticBody;
    if (kind == "kinematic") return b2_kinematicBody;
    if (kind == "dynamic") return b2_dynamicBody;
    throw std::runtime_error("unsupported Phase 9 coupling body kind");
  }

  b2Body& body(const Json& raw_id) {
    const auto found = bodies_.find(raw_id.get<std::string>());
    if (found == bodies_.end()) {
      throw std::runtime_error("Phase 9 coupling body is not live");
    }
    return *found->second;
  }

  void create_body(const Json& raw_id) {
    const auto id = raw_id.get<std::string>();
    const auto& raw = body_declarations_.at(id);
    b2BodyDef definition;
    definition.type = body_type(raw.at("body_kind").get<std::string>());
    definition.position = phase9_vector(raw.at("transform").at("position"));
    definition.angle = float_from_bits(
        raw.at("transform").at("angle_bits").get<std::uint32_t>());
    definition.active = raw.at("active").get<bool>();
    auto* created = world_.CreateBody(&definition);
    if (created == nullptr || !bodies_.emplace(id, created).second) {
      throw std::runtime_error("pinned world failed to create Phase 9 coupling body");
    }
  }

  void create_fixture(const Json& raw_id) {
    const auto id = raw_id.get<std::string>();
    const auto& raw = fixture_declarations_.at(id);
    b2FixtureDef definition;
    b2CircleShape circle;
    b2PolygonShape polygon;
    const auto& shape = raw.at("shape");
    if (shape.at("kind") == "circle") {
      circle.m_p = phase9_vector(shape.at("center"));
      circle.m_radius =
          float_from_bits(shape.at("radius_bits").get<std::uint32_t>());
      definition.shape = &circle;
    } else {
      std::vector<b2Vec2> vertices;
      for (const auto& vertex : shape.at("vertices")) {
        vertices.push_back(phase9_vector(vertex));
      }
      polygon.Set(vertices.data(), static_cast<int32>(vertices.size()));
      definition.shape = &polygon;
    }
    definition.density =
        float_from_bits(raw.at("density_bits").get<std::uint32_t>());
    definition.friction =
        float_from_bits(raw.at("friction_bits").get<std::uint32_t>());
    definition.restitution =
        float_from_bits(raw.at("restitution_bits").get<std::uint32_t>());
    definition.isSensor = raw.at("sensor").get<bool>();
    const auto& filter = raw.at("filter");
    definition.filter.categoryBits = filter.at("category_bits").get<std::uint16_t>();
    definition.filter.maskBits = filter.at("mask_bits").get<std::uint16_t>();
    definition.filter.groupIndex = filter.at("group_index").get<std::int16_t>();
    auto* created = body(raw.at("owner_body_id")).CreateFixture(&definition);
    if (created == nullptr || !fixtures_.emplace(id, created).second) {
      throw std::runtime_error("pinned body failed to create Phase 9 coupling fixture");
    }
  }

  void execute_rigid(const Json& action) {
    const auto kind = action.at("kind").get<std::string>();
    if (kind == "create_body") return create_body(action.at("body_id"));
    if (kind == "create_fixture") return create_fixture(action.at("fixture_id"));
    if (kind == "set_linear_velocity") {
      body(action.at("body_id")).SetLinearVelocity(phase9_vector(action.at("velocity")));
      return;
    }
    if (kind == "set_angular_velocity") {
      body(action.at("body_id")).SetAngularVelocity(float_from_bits(
          action.at("angular_velocity_bits").get<std::uint32_t>()));
      return;
    }
    if (kind == "step") {
      world_.Step(
          float_from_bits(action.at("timestep_bits").get<std::uint32_t>()),
          static_cast<int32>(action.at("velocity_iterations").get<std::uint32_t>()),
          static_cast<int32>(action.at("position_iterations").get<std::uint32_t>()),
          1);
      return discard_dead_particles();
    }
    if (kind == "destroy_fixture") {
      const auto id = action.at("fixture_id").get<std::string>();
      const auto found = fixtures_.find(id);
      if (found != fixtures_.end()) {
        found->second->GetBody()->DestroyFixture(found->second);
        fixtures_.erase(found);
      }
      return;
    }
    if (kind == "destroy_body") {
      const auto id = action.at("body_id").get<std::string>();
      const auto found = bodies_.find(id);
      if (found != bodies_.end()) {
        auto* doomed = found->second;
        for (auto it = fixtures_.begin(); it != fixtures_.end();) {
          it = it->second->GetBody() == doomed ? fixtures_.erase(it) : std::next(it);
        }
        world_.DestroyBody(doomed);
        bodies_.erase(found);
      }
    }
  }

  Json body_snapshots() const {
    Json result = Json::array();
    for (const auto& declaration : timeline_.at("bodies")) {
      const auto id = declaration.at("body_id").get<std::string>();
      const auto found = bodies_.find(id);
      if (found == bodies_.end()) continue;
      const auto& value = *found->second;
      result.push_back(
          {{"body_id", id},
           {"body_kind", rigid_body_kind_name(value.GetType())},
           {"transform", encode_rigid_transform(value)},
           {"active", value.IsActive()},
           {"linear_velocity", encode_rigid_vector(value.GetLinearVelocity())},
           {"angular_velocity_bits", bits_from_float(value.GetAngularVelocity())},
           {"mass_bits", bits_from_float(value.GetMass())},
           {"local_center", encode_rigid_vector(value.GetLocalCenter())},
           {"inertia_bits", bits_from_float(value.GetInertia())}});
    }
    return result;
  }

  void observe_mixed_state() {
    observations_.push_back(
        {{"kind", "particle"},
         {"observation",
          {{"kind", "mixed_state"},
           {"body_ids", Json::array()},
           {"particle_ids", semantic_particle_ids()}}}});
  }

  void create_system(const Json& action) {
    const auto id = action.at("system_id").get<std::string>();
    const auto& raw = system_declarations_.at(id);
    b2ParticleSystemDef definition;
    definition.strictContactCheck = raw.at("strict_contact_check").get<bool>();
    definition.density =
        float_from_bits(raw.at("density_bits").get<std::uint32_t>());
    definition.gravityScale =
        float_from_bits(raw.at("gravity_scale_bits").get<std::uint32_t>());
    definition.radius = float_from_bits(raw.at("radius_bits").get<std::uint32_t>());
    definition.dampingStrength =
        float_from_bits(raw.at("damping_bits").get<std::uint32_t>());
    definition.destroyByAge = raw.at("destruction_by_age").get<bool>();
    definition.lifetimeGranularity =
        float_from_bits(raw.at("lifetime_granularity_bits").get<std::uint32_t>());
    if (!raw.at("maximum_count").is_null()) {
      definition.maxCount = raw.at("maximum_count").get<int32>();
    }
    auto state = std::make_unique<SystemState>();
    state->system = world_.CreateParticleSystem(&definition);
    if (state->system == nullptr) {
      throw std::runtime_error("pinned world failed to create Phase 9 system");
    }
    const auto& buffer = raw.at("buffer_mode");
    state->fixed = buffer.at("kind") == "fixed";
    state->declared_capacity = state->fixed
                                   ? buffer.at("capacity").get<std::size_t>()
                                   : buffer.at("initial_capacity").get<std::size_t>();
    if (state->fixed) {
      const auto capacity = state->declared_capacity;
      state->flags.resize(capacity);
      state->positions.resize(capacity);
      state->velocities.resize(capacity);
      state->colors.resize(capacity);
      state->user_data.resize(capacity);
      state->system->SetFlagsBuffer(state->flags.data(), static_cast<int32>(capacity));
      state->system->SetPositionBuffer(state->positions.data(), static_cast<int32>(capacity));
      state->system->SetVelocityBuffer(state->velocities.data(), static_cast<int32>(capacity));
      state->system->SetColorBuffer(state->colors.data(), static_cast<int32>(capacity));
      state->system->SetUserDataBuffer(state->user_data.data(), static_cast<int32>(capacity));
    }
    state->system->SetPaused(raw.at("paused").get<bool>());
    state->system->SetStuckThreshold(raw.at("stuck_threshold").get<int32>());
    if (!systems_.emplace(id, std::move(state)).second) {
      throw std::runtime_error("duplicate live Phase 9 system");
    }
  }

  void create_particle(const Json& action) {
    const auto id = action.at("particle_id").get<std::string>();
    const auto& raw = particle_declarations_.at(id);
    auto& owner = system(raw.at("system_id"));
    b2ParticleDef definition;
    definition.position = phase9_vector(raw.at("position"));
    definition.velocity = phase9_vector(raw.at("velocity"));
    definition.flags = raw.at("flags_bits").get<uint32>();
    const auto& color = raw.at("color");
    definition.color = b2ParticleColor(
        color.at(0).get<uint8>(), color.at(1).get<uint8>(),
        color.at(2).get<uint8>(), color.at(3).get<uint8>());
    definition.lifetime =
        float_from_bits(raw.at("lifetime_bits").get<std::uint32_t>());
    const auto index = owner.system->CreateParticle(definition);
    if (index == b2_invalidParticleIndex) {
      throw std::runtime_error("pinned system rejected Phase 9 particle creation");
    }
    const auto* handle = owner.system->GetParticleHandleFromIndex(index);
    if (handle == nullptr || !particles_.emplace(id, ParticleState{owner.system, handle}).second) {
      throw std::runtime_error("failed to assign stable Phase 9 particle identity");
    }
  }

  void apply_range(const Json& action, bool impulse) {
    const auto count = action.at("particle_ids").size();
    const auto vector = phase9_vector(action.at(impulse ? "impulse" : "force"));
    const auto distributed = (1.0F / static_cast<float32>(count)) * vector;
    for (const auto& raw_id : action.at("particle_ids")) {
      auto& value = particle(raw_id);
      if (impulse) {
        value.system->ParticleApplyLinearImpulse(value.handle->GetIndex(), distributed);
      } else {
        value.system->ParticleApplyForce(value.handle->GetIndex(), distributed);
      }
    }
  }

  void execute(const Json& action) {
    const auto kind = action.at("kind").get<std::string>();
    if (kind == "create_system") create_system(action);
    else if (kind == "destroy_system") {
      const auto id = action.at("system_id").get<std::string>();
      auto found = systems_.find(id);
      if (found == systems_.end()) throw std::runtime_error("Phase 9 system is not live");
      auto* doomed = found->second->system;
      world_.DestroyParticleSystem(doomed);
      systems_.erase(found);
      for (auto it = particles_.begin(); it != particles_.end();) {
        it = it->second.system == doomed ? particles_.erase(it) : std::next(it);
      }
    } else if (kind == "create_particle") create_particle(action);
    else if (kind == "inspect_system") static_cast<void>(system(action.at("system_id")));
    else if (kind == "inspect_particle") static_cast<void>(particle(action.at("particle_id")));
    else if (kind == "set_paused") {
      system(action.at("system_id")).system->SetPaused(action.at("paused").get<bool>());
    } else if (kind == "set_position") {
      auto& value = particle(action.at("particle_id"));
      value.system->GetPositionBuffer()[value.handle->GetIndex()] =
          phase9_vector(action.at("position"));
    } else if (kind == "set_velocity") {
      auto& value = particle(action.at("particle_id"));
      value.system->GetVelocityBuffer()[value.handle->GetIndex()] =
          phase9_vector(action.at("velocity"));
    } else if (kind == "mark_for_destruction") {
      auto& value = particle(action.at("particle_id"));
      value.system->DestroyParticle(value.handle->GetIndex(), true);
    } else if (kind == "compact") {
      static_cast<void>(system(action.at("system_id")));
      world_.Step(0.0F, 0, 0, 1);
      discard_dead_particles();
    } else if (kind == "apply_force") apply_range(action, false);
    else if (kind == "apply_impulse") apply_range(action, true);
    else if (kind == "request_statistics") observe_statistics(action);
    else if (kind == "query_aabb") observe_query(action);
    else if (kind == "ray_cast") observe_ray(action);
    else throw std::runtime_error("unsupported Phase 9 execution action");
    if (kind != "request_statistics" && kind != "query_aabb" && kind != "ray_cast") {
      observe_mixed_state();
    }
  }

  void observe_statistics(const Json& action) {
    auto& state = system(action.at("system_id"));
    const auto count = state.system->GetParticleCount();
    Json stuck = Json::array();
    for (int32 index = 0; index < state.system->GetStuckCandidateCount(); ++index) {
      const auto dense = state.system->GetStuckCandidates()[index];
      const auto found = std::find_if(particles_.begin(), particles_.end(), [&](const auto& item) {
        return item.second.system == state.system && item.second.handle->GetIndex() == dense;
      });
      if (found != particles_.end()) stuck.push_back(found->first);
    }
    observations_.push_back(
        {{"kind", "particle"},
         {"observation",
          {{"kind", "statistics"},
           {"statistics",
            {{"maybe_system_id", action.at("system_id")},
             {"system_count", static_cast<std::uint32_t>(systems_.size())},
             {"particle_count", static_cast<std::uint32_t>(count)},
             {"pending_particle_count", 0U},
             {"particle_contact_count", static_cast<std::uint32_t>(state.system->GetContactCount())},
             {"body_contact_count", static_cast<std::uint32_t>(state.system->GetBodyContactCount())},
             {"stuck_particle_ids", std::move(stuck)},
             {"collision_energy_bits", bits_from_float(state.system->ComputeCollisionEnergy())},
             {"declared_capacity", static_cast<std::uint32_t>(state.declared_capacity)},
             {"effective_capacity", static_cast<std::uint32_t>(std::max<std::size_t>(state.declared_capacity, static_cast<std::size_t>(count)))}}}}}});
  }

  void observe_query(const Json& action) {
    QueryCollector collector(particles_);
    b2AABB aabb;
    aabb.lowerBound = phase9_vector(action.at("lower"));
    aabb.upperBound = phase9_vector(action.at("upper"));
    if (action.at("system_id").is_null()) {
      world_.QueryAABB(&collector, aabb);
    } else {
      system(action.at("system_id")).system->QueryAABB(&collector, aabb);
    }
    observations_.push_back(
        {{"kind", "particle"},
         {"observation",
          {{"kind", "query"}, {"terminated", false}, {"particle_ids", collector.ids}}}});
  }

  void observe_ray(const Json& action) {
    RayCollector collector(particles_);
    const auto start = phase9_vector(action.at("start"));
    const auto end = phase9_vector(action.at("end"));
    if (action.at("system_id").is_null()) {
      world_.RayCast(&collector, start, end);
    } else {
      system(action.at("system_id")).system->RayCast(&collector, start, end);
    }
    observations_.push_back(
        {{"kind", "particle"},
         {"observation",
          {{"kind", "ray_cast"},
           {"terminated", false},
           {"particle_ids", collector.ids},
           {"fractions_bits", collector.fractions}}}});
  }

  void discard_dead_particles() {
    for (auto it = particles_.begin(); it != particles_.end();) {
      it = it->second.handle == nullptr ||
                   it->second.handle->GetIndex() == b2_invalidParticleIndex
               ? particles_.erase(it)
               : std::next(it);
    }
  }

  b2World& world_;
  Json timeline_;
  std::unordered_map<std::string, Json> body_declarations_;
  std::unordered_map<std::string, Json> fixture_declarations_;
  std::unordered_map<std::string, Json> system_declarations_;
  std::unordered_map<std::string, Json> particle_declarations_;
  std::unordered_map<std::string, std::unique_ptr<SystemState>> systems_;
  std::unordered_map<std::string, ParticleState> particles_;
  std::unordered_map<std::string, b2Body*> bodies_;
  std::unordered_map<std::string, b2Fixture*> fixtures_;
  Json observations_ = Json::array();
  bool phase9_since_checkpoint_ = false;
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
      throw std::runtime_error("Phase 9 checkpoint patch is unmapped");
    }
    (*found)["phase"] = patch.at("phase");
    if (patch.contains("bodies")) (*found)["bodies"] = patch.at("bodies");
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
