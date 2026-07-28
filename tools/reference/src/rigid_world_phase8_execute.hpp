#pragma once

#include "protocol.hpp"
#include "rigid_world_trace.hpp"

#include "nlohmann/json.hpp"

#include <Box2D/Box2D.h>
#include <Box2D/Rope/b2Rope.h>

#include <algorithm>
#include <cstdint>
#include <memory>
#include <stdexcept>
#include <string>
#include <unordered_map>
#include <unordered_set>
#include <utility>
#include <vector>

namespace liquidfun::reference {
namespace phase8_detail {

using Json = nlohmann::json;
inline constexpr std::size_t kMaximumPhase8Observations = 256;

inline b2Vec2 vector(const Json& value) {
  return {
      float_from_bits(value.at("x_bits").get<std::uint32_t>()),
      float_from_bits(value.at("y_bits").get<std::uint32_t>())};
}

inline b2BodyType body_type(std::string_view kind) {
  if (kind == "static") return b2_staticBody;
  if (kind == "kinematic") return b2_kinematicBody;
  if (kind == "dynamic") return b2_dynamicBody;
  throw std::runtime_error("unsupported Phase 8 body kind");
}

inline std::string_view joint_kind_name(b2JointType kind) {
  switch (kind) {
    case e_revoluteJoint: return "revolute";
    case e_prismaticJoint: return "prismatic";
    case e_distanceJoint: return "distance";
    case e_pulleyJoint: return "pulley";
    case e_mouseJoint: return "mouse";
    case e_gearJoint: return "gear";
    case e_wheelJoint: return "wheel";
    case e_weldJoint: return "weld";
    case e_frictionJoint: return "friction";
    case e_ropeJoint: return "rope";
    case e_motorJoint: return "motor";
    case e_unknownJoint: break;
  }
  throw std::runtime_error("unsupported pinned joint kind");
}

inline b2Vec2 semantic_reaction_force(
    const b2Joint& joint,
    float32 inverse_timestep,
    bool solver_initialized) {
  return solver_initialized ? joint.GetReactionForce(inverse_timestep)
                            : b2Vec2{0.0F, 0.0F};
}

inline float32 semantic_reaction_torque(
    const b2Joint& joint,
    float32 inverse_timestep,
    bool solver_initialized) {
  return solver_initialized ? joint.GetReactionTorque(inverse_timestep) : 0.0F;
}

class TimelineExecution final : public b2ContactFilter,
                                public b2ContactListener,
                                public b2DestructionListener {
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
    for (const auto& joint : timeline_.value("joints", Json::array())) {
      joint_declarations_.emplace(joint.at("joint_id").get<std::string>(), joint);
    }
    for (const auto& rope : timeline_.value("ropes", Json::array())) {
      rope_declarations_.emplace(rope.at("rope_id").get<std::string>(), rope);
    }
    world_.SetContactFilter(this);
    world_.SetContactListener(this);
    world_.SetDestructionListener(this);
  }

  ~TimelineExecution() override {
    world_.SetDestructionListener(nullptr);
    world_.SetContactListener(nullptr);
    world_.SetContactFilter(nullptr);
  }

  Json run() {
    Json checkpoints = Json::array();
    std::size_t next_checkpoint = 0;
    for (const auto& record : timeline_.at("actions")) {
      execute(record.at("action"));
      if (next_checkpoint < timeline_.at("checkpoints").size() &&
          timeline_.at("checkpoints").at(next_checkpoint).at("after_action_id") ==
              record.at("action_id")) {
        checkpoints.push_back(capture(timeline_.at("checkpoints").at(next_checkpoint)));
        ++next_checkpoint;
      }
    }
    if (next_checkpoint != timeline_.at("checkpoints").size() || !bodies_.empty() ||
        !fixtures_.empty() || !joints_.empty() || !ropes_.empty()) {
      throw std::runtime_error("Phase 8 timeline did not reset complete state");
    }
    return {
        {"witness_family", timeline_.at("witness_family")},
        {"checkpoints", std::move(checkpoints)}};
  }

 private:
  bool captures_lifecycle() const {
    const auto family = timeline_.at("witness_family").get<std::string>();
    return family == "contact_filter_listener_and_pre_solve_timing" ||
           family == "destruction_listener_and_dependency_cascades";
  }

  bool allows_unconfigured_contacts() const {
    const auto family = timeline_.at("witness_family").get<std::string>();
    return family == "mixed_joint_island_order_and_collision_suppression" ||
           captures_lifecycle();
  }

  void push_entity_lifecycle(std::string_view kind, const std::string& id) {
    if (!captures_lifecycle()) return;
    observations_.push_back(
        {{"kind", "lifecycle"},
         {"event",
          {{"ordinal", next_lifecycle_ordinal_++},
           {"kind", kind},
           {"maybe_contact", nullptr},
           {"maybe_entity_id", id}}}});
  }

  void push_contact_lifecycle(std::string_view kind, b2Contact* contact) {
    if (!captures_lifecycle()) return;
    observations_.push_back(
        {{"kind", "lifecycle"},
         {"event",
          {{"ordinal", next_lifecycle_ordinal_++},
           {"kind", kind},
           {"maybe_contact", contact_identity(contact)},
           {"maybe_entity_id", nullptr}}}});
  }

  bool ShouldCollide(b2Fixture* fixture_a, b2Fixture* fixture_b) override {
    const auto fixture_a_id = semantic_fixture(fixture_a);
    const auto fixture_b_id = semantic_fixture(fixture_b);
    push_entity_lifecycle("filter_decision", fixture_a_id);
    const auto maybe_directive = pair_value(filter_directives_, fixture_a_id, fixture_b_id);
    if (maybe_directive != nullptr) {
      return maybe_directive->at("should_collide").get<bool>();
    }
    return allows_unconfigured_contacts();
  }

  void BeginContact(b2Contact* contact) override {
    if (!seen_contacts_.count(contact)) {
      seen_contacts_.insert(contact);
      push_contact_lifecycle("contact_created", contact);
    }
    push_contact_lifecycle("begin_contact", contact);
  }

  void EndContact(b2Contact* contact) override {
    push_contact_lifecycle("end_contact", contact);
    if (destroying_fixture_or_body_) {
      push_contact_lifecycle("contact_destroyed", contact);
      seen_contacts_.erase(contact);
    }
  }

  void PreSolve(b2Contact* contact, const b2Manifold*) override {
    const auto fixture_a_id = semantic_fixture(contact->GetFixtureA());
    const auto fixture_b_id = semantic_fixture(contact->GetFixtureB());
    const auto maybe_directive =
        pair_value(pre_solve_directives_, fixture_a_id, fixture_b_id);
    if (maybe_directive != nullptr) {
      const auto& directive = maybe_directive->at("directive");
      contact->SetEnabled(directive.at("enabled").get<bool>());
      if (!directive.at("maybe_friction_bits").is_null()) {
        contact->SetFriction(float_bits(directive, "maybe_friction_bits"));
      }
      if (!directive.at("maybe_restitution_bits").is_null()) {
        contact->SetRestitution(float_bits(directive, "maybe_restitution_bits"));
      }
      if (!directive.at("maybe_tangent_speed_bits").is_null()) {
        contact->SetTangentSpeed(float_bits(directive, "maybe_tangent_speed_bits"));
      }
    }
    push_contact_lifecycle("pre_solve", contact);
  }

  void PostSolve(b2Contact* contact, const b2ContactImpulse*) override {
    push_contact_lifecycle("post_solve", contact);
  }

  void SayGoodbye(b2Joint* joint) override {
    const auto maybe_id = maybe_semantic_joint(joint);
    if (maybe_id.empty()) return;
    push_entity_lifecycle("joint_goodbye", maybe_id);
    joints_.erase(maybe_id);
  }

  void SayGoodbye(b2Fixture* fixture) override {
    const auto maybe_id = maybe_semantic_fixture(fixture);
    if (maybe_id.empty()) return;
    push_entity_lifecycle("fixture_goodbye", maybe_id);
    fixtures_.erase(maybe_id);
  }

#include "rigid_world_phase8_execute/actions.hpp"

#include "rigid_world_phase8_execute/joints.hpp"

#include "rigid_world_phase8_execute/capture.hpp"

  b2World& world_;
  Json timeline_;
  std::unordered_map<std::string, Json> body_declarations_;
  std::unordered_map<std::string, Json> fixture_declarations_;
  std::unordered_map<std::string, Json> joint_declarations_;
  std::unordered_map<std::string, Json> rope_declarations_;
  std::unordered_map<std::string, b2Body*> bodies_;
  std::unordered_map<std::string, b2Fixture*> fixtures_;
  std::unordered_map<std::string, b2Joint*> joints_;
  std::unordered_map<std::string, std::unique_ptr<b2Rope>> ropes_;
  std::vector<Json> filter_directives_;
  std::vector<Json> pre_solve_directives_;
  std::unordered_set<const b2Contact*> seen_contacts_;
  Json observations_ = Json::array();
  std::uint32_t next_lifecycle_ordinal_ = 0;
  bool solver_initialized_ = false;
  bool destroying_fixture_or_body_ = false;
};

}  // namespace phase8_detail

inline nlohmann::json execute_phase8_timeline(
    b2World& world,
    std::string_view raw_timeline) {
  return phase8_detail::TimelineExecution(
             world, nlohmann::json::parse(raw_timeline.begin(), raw_timeline.end()))
      .run();
}

}  // namespace liquidfun::reference
