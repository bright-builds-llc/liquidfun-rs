#include "catalog_run_session.hpp"

#include "catalog_checkpoint.hpp"
#include "catalog_joint.hpp"
#include "protocol.hpp"

#include "Box2D/Box2D.h"
#include "Box2D/Rope/b2Rope.h"

#include <algorithm>
#include <cstdint>
#include <memory>
#include <stdexcept>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace liquidfun::reference::catalog_run_detail {

class PrimitiveCounter final : public b2Draw {
 public:
  PrimitiveCounter() {
    SetFlags(e_shapeBit | e_jointBit | e_aabbBit | e_pairBit |
             e_centerOfMassBit | e_particleBit);
  }

  void DrawPolygon(const b2Vec2*, int32, const b2Color&) override { ++count; }
  void DrawSolidPolygon(const b2Vec2*, int32, const b2Color&) override {
    ++count;
  }
  void DrawCircle(const b2Vec2&, float32, const b2Color&) override { ++count; }
  void DrawSolidCircle(
      const b2Vec2&,
      float32,
      const b2Vec2&,
      const b2Color&) override {
    ++count;
  }
  void DrawParticles(
      const b2Vec2*,
      float32,
      const b2ParticleColor*,
      int32 particle_count) override {
    count += static_cast<std::uint32_t>(particle_count);
  }
  void DrawSegment(const b2Vec2&, const b2Vec2&, const b2Color&) override {
    ++count;
  }
  void DrawTransform(const b2Transform&) override { ++count; }

  std::uint32_t count = 0;
};

template <typename Pointer>
const Pointer& lookup(
    const std::vector<std::pair<std::string, Pointer>>& values,
    std::string_view id,
    std::string_view kind) {
  const auto found = std::find_if(
      values.begin(), values.end(),
      [id](const auto& value) { return value.first == id; });
  if (found == values.end()) {
    throw std::runtime_error(std::string("unknown semantic ") +
                             std::string(kind) + " ID");
  }
  return found->second;
}

class CatalogSession {
 public:
  explicit CatalogSession(std::string slug)
      : slug_(std::move(slug)), world_({0.0F, -10.0F}) {}

  void execute(const Json& action, const Json& settings) {
    const auto kind = as_id(member(action, "kind", "catalog action"),
                            "catalog action kind");
    if (kind == "create_body") {
      require_members(action, {"kind", "body_id"}, "create body action");
      create_body(as_id(action.at("body_id"), "body ID"));
    } else if (kind == "create_fixture") {
      require_members(action, {"kind", "fixture_id"}, "create fixture action");
      create_fixture(as_id(action.at("fixture_id"), "fixture ID"));
    } else if (kind == "create_joint") {
      require_members(action, {"kind", "joint_id"}, "create joint action");
      create_joint(as_id(action.at("joint_id"), "joint ID"));
    } else if (kind == "create_rope") {
      require_members(action, {"kind", "rope_id"}, "create rope action");
      create_rope(as_id(action.at("rope_id"), "rope ID"));
    } else if (kind == "particle") {
      require_members(action, {"kind", "action"}, "particle action wrapper");
      execute_particle(action.at("action"));
    } else if (kind == "particle_group") {
      require_members(action, {"kind", "operation"}, "group action wrapper");
      execute_group(action.at("operation"));
    } else if (kind == "configured_step") {
      require_members(action,
                      {"kind", "timestep_bits", "velocity_iterations",
                       "position_iterations", "continuous_work_budget"},
                      "configured step action");
      if (as_u32(action.at("continuous_work_budget"),
                 "continuous work budget") != 1U) {
        throw std::runtime_error("unsupported continuous work budget");
      }
      step(action, settings);
    } else {
      execute_mutation(kind, action);
    }
  }

  CatalogCheckpointInput checkpoint(
      const CatalogRequest& request,
      const Json& declaration) {
    PrimitiveCounter draw;
    world_.SetDebugDraw(&draw);
    world_.DrawDebugData();
    WorldCounts counts;
    counts.bodies = static_cast<std::uint32_t>(world_.GetBodyCount());
    counts.contacts = static_cast<std::uint32_t>(world_.GetContactCount());
    counts.joints = static_cast<std::uint32_t>(world_.GetJointCount());
    for (auto* system = world_.GetParticleSystemList(); system != nullptr;
         system = system->GetNext()) {
      ++counts.particle_systems;
      counts.particles += static_cast<std::uint32_t>(system->GetParticleCount());
      for (auto* group = system->GetParticleGroupList(); group != nullptr;
           group = group->GetNext()) {
        ++counts.particle_groups;
      }
    }
    for (auto* body = world_.GetBodyList(); body != nullptr;
         body = body->GetNext()) {
      for (auto* fixture = body->GetFixtureList(); fixture != nullptr;
           fixture = fixture->GetNext()) {
        ++counts.fixtures;
      }
    }
    return {request.request_id,
            request.resolved_sha256,
            as_id(declaration.at("checkpoint_id"), "checkpoint ID"),
            as_u32(declaration.at("logical_step"), "logical step"),
            bits_from_float(simulation_time_),
            counts,
            draw.count};
  }

 private:
  void create_body(const std::string& id) {
    if (std::any_of(bodies_.begin(), bodies_.end(),
                    [&id](const auto& item) { return item.first == id; })) {
      throw std::runtime_error("duplicate semantic body ID");
    }
    b2BodyDef definition;
    definition.type = b2_dynamicBody;
    definition.position.Set(static_cast<float>(bodies_.size()) * 0.75F, 0.0F);
    bodies_.emplace_back(id, world_.CreateBody(&definition));
  }

  void create_fixture(const std::string& id) {
    if (bodies_.empty()) {
      throw std::runtime_error("fixture requires a body");
    }
    b2CircleShape shape;
    shape.m_radius = 0.5F;
    b2FixtureDef definition;
    definition.shape = &shape;
    definition.density = 1.0F;
    definition.friction = 0.3F;
    auto* body = bodies_.at(fixtures_.size() % bodies_.size()).second;
    fixtures_.emplace_back(id, body->CreateFixture(&definition));
  }

  void create_joint(const std::string& id) {
    std::vector<b2Body*> bodies;
    bodies.reserve(bodies_.size());
    for (const auto& item : bodies_) bodies.push_back(item.second);
    std::vector<b2Joint*> joints;
    joints.reserve(joints_.size());
    for (const auto& item : joints_) joints.push_back(item.second);
    joints_.emplace_back(
        id, create_catalog_joint(slug_, bodies, joints, world_));
  }

  void create_rope(const std::string& id) {
    auto rope = std::make_unique<b2Rope>();
    std::vector<b2Vec2> vertices{{0.0F, 2.0F}, {0.0F, 1.0F}, {0.0F, 0.0F}};
    std::vector<float32> masses{0.0F, 1.0F, 1.0F};
    b2RopeDef definition;
    definition.vertices = vertices.data();
    definition.masses = masses.data();
    definition.count = static_cast<int32>(vertices.size());
    definition.gravity.Set(0.0F, -10.0F);
    definition.damping = 0.1F;
    definition.k2 = 1.0F;
    definition.k3 = 0.5F;
    rope->Initialize(&definition);
    ropes_.emplace_back(id, std::move(rope));
  }

  void step(const Json& action, const Json& settings) {
    const auto timestep =
        as_finite_float(action.at("timestep_bits"), "step timestep");
    const auto velocity = as_u32(action.at("velocity_iterations"), "velocity iterations");
    const auto position = as_u32(action.at("position_iterations"), "position iterations");
    const auto particles = settings.at("particle_iterations").get<std::uint32_t>();
    if (timestep <= 0.0F || velocity == 0U || position == 0U || particles == 0U) {
      throw std::runtime_error("invalid catalog step");
    }
    world_.Step(timestep, static_cast<int32>(velocity),
                static_cast<int32>(position), static_cast<int32>(particles));
    simulation_time_ += timestep;
  }

  void execute_mutation(const std::string& kind, const Json& action);
  void execute_particle(const Json& action);
  void execute_group(const Json& operation);

  std::string slug_;
  b2World world_;
  float simulation_time_ = 0.0F;
  std::vector<std::pair<std::string, b2Body*>> bodies_;
  std::vector<std::pair<std::string, b2Fixture*>> fixtures_;
  std::vector<std::pair<std::string, b2Joint*>> joints_;
  std::vector<std::pair<std::string, std::unique_ptr<b2Rope>>> ropes_;
  std::vector<std::pair<std::string, b2ParticleSystem*>> systems_;
  std::vector<std::pair<std::string, b2ParticleHandle*>> particles_;
  std::vector<std::pair<std::string, b2ParticleGroup*>> groups_;
};

void validate_schedule(const CatalogRequest& request) {
  const auto& payload = request.payload;
  const auto& actions = payload.at("actions");
  const auto& checkpoints = payload.at("checkpoints");
  std::size_t checkpoint_index = 0;
  bool saw_logical = false;
  for (std::size_t index = 0; index < actions.size(); ++index) {
    const auto& scheduled = actions.at(index);
    require_members(
        scheduled, {"action_id", "schedule", "action"}, "scheduled action");
    const auto digits = std::to_string(index);
    const auto padding = 4U - std::min<std::size_t>(4U, digits.size());
    const auto expected_id = "action-" + std::string(padding, '0') + digits;
    if (as_id(scheduled.at("action_id"), "action ID") != expected_id) {
      throw std::runtime_error("catalog action order is invalid");
    }
    const auto& schedule = scheduled.at("schedule");
    const auto schedule_kind = as_id(schedule.at("kind"), "schedule kind");
    if (schedule_kind == "setup") {
      if (saw_logical ||
          as_u32(schedule.at("ordinal"), "setup ordinal") != index) {
        throw std::runtime_error("catalog setup order is invalid");
      }
      continue;
    }
    if (schedule_kind != "logical_step") {
      throw std::runtime_error("unknown catalog schedule kind");
    }
    saw_logical = true;
    const auto logical_step =
        as_u32(schedule.at("ordinal"), "logical step");
    if (logical_step != checkpoint_index + 1U ||
        checkpoint_index >= checkpoints.size()) {
      throw std::runtime_error("catalog logical step order is invalid");
    }
    const auto& declaration = checkpoints.at(checkpoint_index);
    require_members(
        declaration,
        {"checkpoint_id", "after_action_id", "logical_step"},
        "checkpoint declaration");
    if (declaration.at("after_action_id") != scheduled.at("action_id") ||
        as_u32(
            declaration.at("logical_step"), "checkpoint logical step") !=
            logical_step) {
      throw std::runtime_error("catalog checkpoint reference is invalid");
    }
    ++checkpoint_index;
  }
  if (!saw_logical || checkpoint_index != checkpoints.size()) {
    throw std::runtime_error("catalog checkpoint schedule is incomplete");
  }
}

class CatalogExecutionSession::Impl {
 public:
  explicit Impl(const CatalogRequest& source)
      : request(source),
        session(as_id(
            request.payload.at("identity").at("slug"), "catalog slug")) {
    validate_schedule(request);
    const auto& actions = request.payload.at("actions");
    const auto& settings = request.payload.at("identity").at("settings");
    while (next_action < actions.size() &&
           actions.at(next_action).at("schedule").at("kind") == "setup") {
      session.execute(actions.at(next_action).at("action"), settings);
      ++next_action;
    }
  }

  CatalogRequest request;
  CatalogSession session;
  std::size_t next_action = 0;
  std::size_t completed_logical_actions = 0;
};

CatalogExecutionSession::CatalogExecutionSession(
    const CatalogRequest& request)
    : impl_(std::make_unique<Impl>(request)) {}

CatalogExecutionSession::~CatalogExecutionSession() = default;

CatalogExecutionSession::CatalogExecutionSession(
    CatalogExecutionSession&&) noexcept = default;

CatalogExecutionSession& CatalogExecutionSession::operator=(
    CatalogExecutionSession&&) noexcept = default;

std::size_t CatalogExecutionSession::logical_action_count() const {
  return impl_->request.payload.at("checkpoints").size();
}

bool CatalogExecutionSession::finished() const {
  return impl_->next_action == impl_->request.payload.at("actions").size();
}

void CatalogExecutionSession::execute_next_logical_action() {
  if (finished()) {
    throw std::runtime_error("catalog logical action stream is exhausted");
  }
  const auto& scheduled =
      impl_->request.payload.at("actions").at(impl_->next_action);
  if (scheduled.at("schedule").at("kind") != "logical_step") {
    throw std::runtime_error("catalog logical action stream is invalid");
  }
  impl_->session.execute(
      scheduled.at("action"),
      impl_->request.payload.at("identity").at("settings"));
  ++impl_->next_action;
  ++impl_->completed_logical_actions;
}

std::string CatalogExecutionSession::capture_current_checkpoint() const {
  if (impl_->completed_logical_actions == 0U) {
    throw std::runtime_error(
        "catalog checkpoint capture requires a logical action");
  }
  const auto checkpoint_index = impl_->completed_logical_actions - 1U;
  return encode_catalog_checkpoint(impl_->session.checkpoint(
      impl_->request,
      impl_->request.payload.at("checkpoints").at(checkpoint_index)));
}

void CatalogSession::execute_mutation(
    const std::string& kind,
    const Json& action) {
  if (kind == "set_world_gravity") {
    world_.SetGravity(as_vec2(action.at("gravity"), "gravity"));
  } else if (kind == "inspect_body") {
    static_cast<void>(lookup(bodies_, as_id(action.at("body_id"), "body ID"), "body"));
  } else if (kind == "inspect_fixture") {
    static_cast<void>(lookup(fixtures_, as_id(action.at("fixture_id"), "fixture ID"), "fixture"));
  } else if (kind == "inspect_joint") {
    static_cast<void>(lookup(joints_, as_id(action.at("joint_id"), "joint ID"), "joint"));
  } else if (kind == "inspect_rope") {
    static_cast<void>(lookup(ropes_, as_id(action.at("rope_id"), "rope ID"), "rope"));
  } else if (kind == "set_linear_velocity") {
    lookup(bodies_, as_id(action.at("body_id"), "body ID"), "body")
        ->SetLinearVelocity(as_vec2(action.at("velocity"), "velocity"));
  } else if (kind == "apply_force") {
    lookup(bodies_, as_id(action.at("body_id"), "body ID"), "body")
        ->ApplyForce(as_vec2(action.at("force"), "force"),
                     as_vec2(action.at("point"), "force point"), true);
  } else if (kind == "set_awake") {
    lookup(bodies_, as_id(action.at("body_id"), "body ID"), "body")
        ->SetAwake(action.at("awake").get<bool>());
  } else if (kind == "set_sleeping_allowed") {
    lookup(bodies_, as_id(action.at("body_id"), "body ID"), "body")
        ->SetSleepingAllowed(action.at("sleeping_allowed").get<bool>());
  } else if (kind == "set_bullet") {
    lookup(bodies_, as_id(action.at("body_id"), "body ID"), "body")
        ->SetBullet(action.at("bullet").get<bool>());
  } else if (kind == "set_body_type") {
    const auto body_kind = action.at("body_kind").get<std::string>();
    const auto type = body_kind == "static" ? b2_staticBody
                      : body_kind == "kinematic" ? b2_kinematicBody
                      : body_kind == "dynamic"   ? b2_dynamicBody
                                                   : b2BodyType(-1);
    if (static_cast<int>(type) < 0) {
      throw std::runtime_error("unknown body kind");
    }
    lookup(bodies_, as_id(action.at("body_id"), "body ID"), "body")
        ->SetType(type);
  } else if (kind == "set_fixture_sensor") {
    lookup(fixtures_, as_id(action.at("fixture_id"), "fixture ID"), "fixture")
        ->SetSensor(action.at("sensor").get<bool>());
  } else if (kind == "set_fixture_material") {
    auto* fixture = lookup(fixtures_, as_id(action.at("fixture_id"), "fixture ID"), "fixture");
    fixture->SetFriction(as_finite_float(action.at("friction_bits"), "friction"));
    fixture->SetRestitution(as_finite_float(action.at("restitution_bits"), "restitution"));
  } else if (kind == "set_fixture_filter") {
    auto* fixture = lookup(fixtures_, as_id(action.at("fixture_id"), "fixture ID"), "fixture");
    const auto& filter_json = action.at("filter");
    b2Filter filter;
    filter.categoryBits = static_cast<std::uint16_t>(
        as_u32(filter_json.at("category_bits"), "category bits"));
    filter.maskBits = static_cast<std::uint16_t>(
        as_u32(filter_json.at("mask_bits"), "mask bits"));
    filter.groupIndex = filter_json.at("group_index").get<std::int16_t>();
    fixture->SetFilterData(filter);
  } else if (kind == "set_continuous_physics") {
    world_.SetContinuousPhysics(action.at("enabled").get<bool>());
  } else if (kind == "mutate_joint") {
    mutate_catalog_joint(
        *lookup(joints_, as_id(action.at("joint_id"), "joint ID"), "joint"),
        action.at("mutation"));
  } else if (kind == "set_rope_angle") {
    lookup(ropes_, as_id(action.at("rope_id"), "rope ID"), "rope")
        ->SetAngle(as_finite_float(action.at("angle_bits"), "rope angle"));
  } else if (kind == "step_rope") {
    const auto timestep = as_finite_float(action.at("timestep_bits"), "rope timestep");
    const auto iterations = as_u32(action.at("iterations"), "rope iterations");
    lookup(ropes_, as_id(action.at("rope_id"), "rope ID"), "rope")
        ->Step(timestep, static_cast<int32>(iterations));
    simulation_time_ += timestep;
  } else if (kind == "query_aabb" || kind == "ray_cast" ||
             kind == "set_contact_filter_directive" ||
             kind == "set_pre_solve_directive") {
    // Definitions are fully validated by the Rust resolver. The catalog
    // oracle keeps these callback policies semantic and never exposes native
    // proxy or callback storage as identity.
  } else if (kind == "destroy_fixture") {
    const auto id = as_id(action.at("fixture_id"), "fixture ID");
    auto* fixture = lookup(fixtures_, id, "fixture");
    fixture->GetBody()->DestroyFixture(fixture);
    fixtures_.erase(std::remove_if(fixtures_.begin(), fixtures_.end(),
                                   [&id](const auto& item) { return item.first == id; }),
                    fixtures_.end());
  } else if (kind == "destroy_body") {
    const auto id = as_id(action.at("body_id"), "body ID");
    auto* body = lookup(bodies_, id, "body");
    world_.DestroyBody(body);
    bodies_.erase(std::remove_if(bodies_.begin(), bodies_.end(),
                                [&id](const auto& item) { return item.first == id; }),
                 bodies_.end());
  } else {
    throw std::runtime_error("unsupported catalog action kind");
  }
}

void CatalogSession::execute_particle(const Json& action) {
  const auto kind = as_id(action.at("kind"), "particle action kind");
  if (kind == "create_system") {
    b2ParticleSystemDef definition;
    systems_.emplace_back(as_id(action.at("system_id"), "system ID"),
                          world_.CreateParticleSystem(&definition));
  } else if (kind == "destroy_system") {
    const auto id = as_id(action.at("system_id"), "system ID");
    auto* system = lookup(systems_, id, "particle system");
    world_.DestroyParticleSystem(system);
    systems_.erase(std::remove_if(systems_.begin(), systems_.end(),
                                  [&id](const auto& item) { return item.first == id; }),
                   systems_.end());
    particles_.clear();
    groups_.clear();
  } else if (kind == "create_particle") {
    if (systems_.empty()) {
      throw std::runtime_error("particle requires a system");
    }
    b2ParticleDef definition;
    const auto index = systems_.front().second->CreateParticle(definition);
    particles_.emplace_back(
        as_id(action.at("particle_id"), "particle ID"),
        const_cast<b2ParticleHandle*>(
            systems_.front().second->GetParticleHandleFromIndex(index)));
  } else if (kind == "set_paused") {
    lookup(systems_, as_id(action.at("system_id"), "system ID"), "particle system")
        ->SetPaused(action.at("paused").get<bool>());
  } else if (kind == "set_position" || kind == "set_velocity") {
    auto* handle = lookup(particles_, as_id(action.at("particle_id"), "particle ID"), "particle");
    if (handle->GetIndex() < 0 || systems_.empty()) {
      throw std::runtime_error("stale semantic particle ID");
    }
    auto* buffer = kind == "set_position" ? systems_.front().second->GetPositionBuffer()
                                           : systems_.front().second->GetVelocityBuffer();
    buffer[handle->GetIndex()] = as_vec2(
        action.at(kind == "set_position" ? "position" : "velocity"), kind);
  } else if (kind == "mark_for_destruction") {
    auto* handle = lookup(particles_, as_id(action.at("particle_id"), "particle ID"), "particle");
    systems_.front().second->DestroyParticle(handle->GetIndex());
  } else if (kind == "compact") {
    world_.Step(0.0F, 1, 1, 1);
  } else if (kind == "apply_force" || kind == "apply_impulse") {
    const auto force = as_vec2(action.at(kind == "apply_force" ? "force" : "impulse"), kind);
    for (const auto& id_value : action.at("particle_ids")) {
      auto* handle = lookup(particles_, as_id(id_value, "particle ID"), "particle");
      if (kind == "apply_force") {
        systems_.front().second->ParticleApplyForce(handle->GetIndex(), force);
      } else {
        systems_.front().second->ParticleApplyLinearImpulse(handle->GetIndex(), force);
      }
    }
  } else if (kind == "inspect_system" || kind == "request_statistics") {
    static_cast<void>(lookup(
        systems_, as_id(action.at("system_id"), "system ID"),
        "particle system"));
  } else if (kind == "inspect_particle") {
    static_cast<void>(lookup(
        particles_, as_id(action.at("particle_id"), "particle ID"),
        "particle"));
  } else if (kind == "inspect_particle_contact" || kind == "inspect_body_contact" ||
             kind == "inspect_occurrence" || kind == "query_aabb" ||
             kind == "ray_cast") {
    // These actions observe bounded public semantics without exporting the
    // upstream callback index that selected the record.
  } else {
    throw std::runtime_error("unsupported particle action kind");
  }
}

void CatalogSession::execute_group(const Json& operation) {
  const auto kind = as_id(operation.at("kind"), "group action kind");
  if (kind == "create_group") {
    const auto& definition_json = operation.at("definition");
    auto* system = lookup(
        systems_, as_id(definition_json.at("system_id"), "system ID"),
        "particle system");
    std::vector<b2Vec2> positions;
    const auto& source = definition_json.at("source");
    if (source.at("kind") != "explicit") {
      throw std::runtime_error("unsupported particle group source");
    }
    for (const auto& position : source.at("positions")) {
      positions.push_back(as_vec2(position, "group position"));
    }
    if (positions.size() != definition_json.at("member_ids").size() ||
        positions.empty()) {
      throw std::runtime_error("particle group member mismatch");
    }
    b2ParticleGroupDef definition;
    definition.flags = as_u32(definition_json.at("particle_flags_bits"), "particle flags");
    definition.groupFlags = as_u32(definition_json.at("group_flags_bits"), "group flags");
    definition.particleCount = static_cast<int32>(positions.size());
    definition.positionData = positions.data();
    const auto& destination = definition_json.at("destination");
    if (destination.at("kind") == "append_to") {
      definition.group = lookup(
          groups_,
          as_id(destination.at("target_group_id"), "target group ID"),
          "particle group");
    } else if (destination.at("kind") != "new") {
      throw std::runtime_error("unknown group destination");
    }
    auto* group = system->CreateParticleGroup(definition);
    const auto& member_ids = definition_json.at("member_ids");
    const auto first_index = group->GetBufferIndex() + group->GetParticleCount() -
                             static_cast<int32>(member_ids.size());
    for (std::size_t index = 0; index < member_ids.size(); ++index) {
      particles_.emplace_back(
          as_id(member_ids.at(index), "particle ID"),
          const_cast<b2ParticleHandle*>(system->GetParticleHandleFromIndex(
              first_index + static_cast<int32>(index))));
    }
    if (destination.at("kind") == "new") {
      groups_.emplace_back(as_id(definition_json.at("group_id"), "group ID"), group);
    }
  } else if (kind == "join_groups") {
    auto* target = lookup(
        groups_, as_id(operation.at("target_group_id"), "target group ID"),
        "particle group");
    auto* source = lookup(
        groups_, as_id(operation.at("source_group_id"), "source group ID"),
        "particle group");
    target->GetParticleSystem()->JoinParticleGroups(target, source);
  } else if (kind == "split_group") {
    auto* group = lookup(groups_, as_id(operation.at("group_id"), "group ID"), "particle group");
    auto* system = group->GetParticleSystem();
    system->SplitParticleGroup(group);
    auto* candidate = system->GetParticleGroupList();
    for (const auto& id : operation.at("created_group_ids")) {
      while (candidate != nullptr &&
             std::any_of(groups_.begin(), groups_.end(), [candidate](const auto& item) {
               return item.second == candidate;
             })) {
        candidate = candidate->GetNext();
      }
      if (candidate == nullptr) {
        break;
      }
      groups_.emplace_back(as_id(id, "created group ID"), candidate);
      candidate = candidate->GetNext();
    }
  } else if (kind == "set_group_flags") {
    lookup(groups_, as_id(operation.at("group_id"), "group ID"), "particle group")
        ->SetGroupFlags(as_u32(operation.at("group_flags_bits"), "group flags"));
  } else if (kind == "destroy_group") {
    const auto id = as_id(operation.at("group_id"), "group ID");
    auto* group = lookup(groups_, id, "particle group");
    group->DestroyParticles(false);
  } else if (kind == "step") {
    const auto timestep = as_finite_float(operation.at("timestep_bits"), "group timestep");
    world_.Step(
        timestep,
        static_cast<int32>(as_u32(
            operation.at("velocity_iterations"), "velocity iterations")),
        static_cast<int32>(as_u32(
            operation.at("position_iterations"), "position iterations")),
        static_cast<int32>(as_u32(
            operation.at("particle_iterations"), "particle iterations")));
    simulation_time_ += timestep;
  } else if (kind != "inspect_state") {
    throw std::runtime_error("unsupported particle group action kind");
  }
}

std::vector<std::string> execute_payload(const CatalogRequest& request) {
  CatalogExecutionSession session(request);
  std::vector<std::string> records;
  records.reserve(session.logical_action_count());
  while (!session.finished()) {
    session.execute_next_logical_action();
    records.push_back(session.capture_current_checkpoint());
  }
  return records;
}

}  // namespace liquidfun::reference::catalog_run_detail
