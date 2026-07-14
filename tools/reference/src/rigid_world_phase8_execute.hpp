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

  void execute(const Json& action) {
    const auto kind = action.at("kind").get<std::string>();
    if (kind == "create_body") return create_body(action.at("body_id"));
    if (kind == "create_fixture") return create_fixture(action.at("fixture_id"));
    if (kind == "create_joint") return create_joint(action.at("joint_id"));
    if (kind == "inspect_joint") return observe_joint(action.at("joint_id"));
    if (kind == "mutate_joint") {
      mutate_joint(action.at("joint_id"), action.at("mutation"));
      return observe_joint(action.at("joint_id"));
    }
    if (kind == "destroy_joint") return destroy_joint(action.at("joint_id"));
    if (kind == "create_rope") return create_rope(action.at("rope_id"));
    if (kind == "set_rope_angle") {
      rope_json(action.at("rope_id")).SetAngle(float_bits(action, "angle_bits"));
      return observe_rope(action.at("rope_id"));
    }
    if (kind == "step_rope") {
      rope_json(action.at("rope_id")).Step(
          float_bits(action, "timestep_bits"),
          static_cast<int32>(action.at("iterations").get<std::uint32_t>()));
      return observe_rope(action.at("rope_id"));
    }
    if (kind == "inspect_rope") return observe_rope(action.at("rope_id"));
    if (kind == "destroy_rope") {
      ropes_.erase(action.at("rope_id").get<std::string>());
      return;
    }
    if (kind == "request_reconstruction") return reconstruct();
    if (kind == "request_diagnostics") return diagnostics();
    if (kind == "set_contact_filter_directive") {
      filter_directives_.push_back(action);
      refilter(action.at("target"));
      return;
    }
    if (kind == "set_pre_solve_directive") {
      pre_solve_directives_.push_back(action);
      return;
    }
    if (kind == "set_linear_velocity") {
      const auto id = action.at("body_id").get<std::string>();
      body(id).SetLinearVelocity(vector(action.at("velocity")));
      return observe_body(id);
    }
    if (kind == "inspect_body") return;
    if (kind == "step") return step(action);
    if (kind == "destroy_fixture") return destroy_fixture(action.at("fixture_id"));
    if (kind == "destroy_body") return destroy_body(action.at("body_id"));
    throw std::runtime_error("unsupported Phase 8 execution action");
  }

  static float32 float_bits(const Json& value, std::string_view name) {
    return float_from_bits(value.at(name).get<std::uint32_t>());
  }

  static const Json* pair_value(
      const std::vector<Json>& directives,
      const std::string& fixture_a_id,
      const std::string& fixture_b_id) {
    const auto found = std::find_if(
        directives.rbegin(), directives.rend(), [&](const auto& directive) {
          const auto& target = directive.at("target");
          const auto target_a =
              target.at("fixture_a_id").template get<std::string>();
          const auto target_b =
              target.at("fixture_b_id").template get<std::string>();
          return (target_a == fixture_a_id && target_b == fixture_b_id) ||
                 (target_a == fixture_b_id && target_b == fixture_a_id);
        });
    return found == directives.rend() ? nullptr : &*found;
  }

  void refilter(const Json& target) {
    for (const auto* name : {"fixture_a_id", "fixture_b_id"}) {
      auto& value = fixture(target.at(name));
      value.SetFilterData(value.GetFilterData());
    }
  }

  void step(const Json& action) {
    auto& contact_manager =
        const_cast<b2ContactManager&>(world_.GetContactManager());
    contact_manager.FindNewContacts();
    world_.Step(
        float_bits(action, "timestep_bits"),
        static_cast<int32>(action.at("velocity_iterations").get<std::uint32_t>()),
        static_cast<int32>(action.at("position_iterations").get<std::uint32_t>()),
        1);
    solver_initialized_ = true;
  }

  void observe_body(const std::string& id) {
    const auto& value = body(id);
    observations_.push_back(
        {{"kind", "body_state"},
         {"state",
          {{"body_id", id},
           {"linear_velocity", encode_rigid_vector(value.GetLinearVelocity())},
           {"angular_velocity_bits", bits_from_float(value.GetAngularVelocity())},
           {"awake", value.IsAwake()},
           {"bullet", value.IsBullet()},
           {"sleeping_allowed", value.IsSleepingAllowed()},
           {"fixed_rotation", value.IsFixedRotation()},
           {"linear_damping_bits", bits_from_float(value.GetLinearDamping())},
           {"angular_damping_bits", bits_from_float(value.GetAngularDamping())},
           {"gravity_scale_bits", bits_from_float(value.GetGravityScale())}}}});
  }

  void create_body(const Json& raw_id) {
    const auto id = raw_id.get<std::string>();
    const auto& declaration = body_declarations_.at(id);
    b2BodyDef definition;
    definition.type = body_type(declaration.at("body_kind").get<std::string>());
    definition.position = vector(declaration.at("transform").at("position"));
    definition.angle = float_bits(declaration.at("transform"), "angle_bits");
    definition.active = declaration.at("active").get<bool>();
    auto* created = world_.CreateBody(&definition);
    if (created == nullptr || !bodies_.emplace(id, created).second) {
      throw std::runtime_error("pinned world failed to create Phase 8 body");
    }
  }

  void create_fixture(const Json& raw_id) {
    const auto id = raw_id.get<std::string>();
    const auto& declaration = fixture_declarations_.at(id);
    b2FixtureDef definition;
    b2CircleShape circle;
    b2PolygonShape polygon;
    const auto& raw_shape = declaration.at("shape");
    if (raw_shape.at("kind") == "circle") {
      circle.m_p = vector(raw_shape.at("center"));
      circle.m_radius = float_bits(raw_shape, "radius_bits");
      definition.shape = &circle;
    } else {
      std::vector<b2Vec2> vertices;
      for (const auto& vertex : raw_shape.at("vertices")) {
        vertices.push_back(vector(vertex));
      }
      polygon.Set(vertices.data(), static_cast<int32>(vertices.size()));
      definition.shape = &polygon;
    }
    definition.density = float_bits(declaration, "density_bits");
    definition.friction = float_bits(declaration, "friction_bits");
    definition.restitution = float_bits(declaration, "restitution_bits");
    definition.isSensor = declaration.at("sensor").get<bool>();
    const auto& raw_filter = declaration.at("filter");
    definition.filter.categoryBits = raw_filter.at("category_bits").get<std::uint16_t>();
    definition.filter.maskBits = raw_filter.at("mask_bits").get<std::uint16_t>();
    definition.filter.groupIndex = raw_filter.at("group_index").get<std::int16_t>();
    auto* created = body(declaration.at("owner_body_id")).CreateFixture(&definition);
    if (created == nullptr || !fixtures_.emplace(id, created).second) {
      throw std::runtime_error("pinned body failed to create Phase 8 fixture");
    }
  }

  template <typename Definition>
  b2Joint* create_typed_joint(
      Definition& definition,
      const Json& declaration) {
    definition.bodyA = &body(declaration.at("body_a_id"));
    definition.bodyB = &body(declaration.at("body_b_id"));
    definition.collideConnected = declaration.at("collide_connected").get<bool>();
    auto* created = world_.CreateJoint(&definition);
    if (created == nullptr) throw std::runtime_error("pinned world failed to create joint");
    return created;
  }

  void create_joint(const Json& raw_id) {
    const auto id = raw_id.get<std::string>();
    const auto& declaration = joint_declarations_.at(id);
    const auto& value = declaration.at("definition");
    const auto kind = value.at("kind").get<std::string>();
    b2Joint* created = nullptr;
    if (kind == "revolute") {
      b2RevoluteJointDef definition;
      definition.localAnchorA = vector(value.at("local_anchor_a"));
      definition.localAnchorB = vector(value.at("local_anchor_b"));
      definition.referenceAngle = float_bits(value, "reference_angle_bits");
      definition.lowerAngle = float_bits(value, "lower_angle_bits");
      definition.upperAngle = float_bits(value, "upper_angle_bits");
      definition.motorSpeed = float_bits(value, "motor_speed_bits");
      definition.maxMotorTorque = float_bits(value, "max_motor_torque_bits");
      definition.enableLimit = value.at("limit_enabled").get<bool>();
      definition.enableMotor = value.at("motor_enabled").get<bool>();
      created = create_typed_joint(definition, declaration);
    } else if (kind == "prismatic") {
      b2PrismaticJointDef definition;
      definition.localAnchorA = vector(value.at("local_anchor_a"));
      definition.localAnchorB = vector(value.at("local_anchor_b"));
      definition.localAxisA = vector(value.at("local_axis_a"));
      definition.referenceAngle = float_bits(value, "reference_angle_bits");
      definition.lowerTranslation = float_bits(value, "lower_translation_bits");
      definition.upperTranslation = float_bits(value, "upper_translation_bits");
      definition.motorSpeed = float_bits(value, "motor_speed_bits");
      definition.maxMotorForce = float_bits(value, "max_motor_force_bits");
      definition.enableLimit = value.at("limit_enabled").get<bool>();
      definition.enableMotor = value.at("motor_enabled").get<bool>();
      created = create_typed_joint(definition, declaration);
    } else if (kind == "distance") {
      b2DistanceJointDef definition;
      definition.localAnchorA = vector(value.at("local_anchor_a"));
      definition.localAnchorB = vector(value.at("local_anchor_b"));
      definition.length = float_bits(value, "length_bits");
      definition.frequencyHz = float_bits(value, "frequency_bits");
      definition.dampingRatio = float_bits(value, "damping_ratio_bits");
      created = create_typed_joint(definition, declaration);
    } else if (kind == "pulley") {
      b2PulleyJointDef definition;
      definition.groundAnchorA = vector(value.at("ground_anchor_a"));
      definition.groundAnchorB = vector(value.at("ground_anchor_b"));
      definition.localAnchorA = vector(value.at("local_anchor_a"));
      definition.localAnchorB = vector(value.at("local_anchor_b"));
      definition.lengthA = float_bits(value, "length_a_bits");
      definition.lengthB = float_bits(value, "length_b_bits");
      definition.ratio = float_bits(value, "ratio_bits");
      created = create_typed_joint(definition, declaration);
    } else if (kind == "mouse") {
      b2MouseJointDef definition;
      definition.target = vector(value.at("target"));
      definition.maxForce = float_bits(value, "max_force_bits");
      definition.frequencyHz = float_bits(value, "frequency_bits");
      definition.dampingRatio = float_bits(value, "damping_ratio_bits");
      created = create_typed_joint(definition, declaration);
    } else if (kind == "gear") {
      b2GearJointDef definition;
      definition.joint1 = &joint_json(value.at("joint_a_id"));
      definition.joint2 = &joint_json(value.at("joint_b_id"));
      definition.ratio = float_bits(value, "ratio_bits");
      created = create_typed_joint(definition, declaration);
    } else if (kind == "wheel") {
      b2WheelJointDef definition;
      definition.localAnchorA = vector(value.at("local_anchor_a"));
      definition.localAnchorB = vector(value.at("local_anchor_b"));
      definition.localAxisA = vector(value.at("local_axis_a"));
      definition.motorSpeed = float_bits(value, "motor_speed_bits");
      definition.maxMotorTorque = float_bits(value, "max_motor_torque_bits");
      definition.frequencyHz = float_bits(value, "frequency_bits");
      definition.dampingRatio = float_bits(value, "damping_ratio_bits");
      definition.enableMotor = value.at("motor_enabled").get<bool>();
      created = create_typed_joint(definition, declaration);
    } else if (kind == "weld") {
      b2WeldJointDef definition;
      definition.localAnchorA = vector(value.at("local_anchor_a"));
      definition.localAnchorB = vector(value.at("local_anchor_b"));
      definition.referenceAngle = float_bits(value, "reference_angle_bits");
      definition.frequencyHz = float_bits(value, "frequency_bits");
      definition.dampingRatio = float_bits(value, "damping_ratio_bits");
      created = create_typed_joint(definition, declaration);
    } else if (kind == "friction") {
      b2FrictionJointDef definition;
      definition.localAnchorA = vector(value.at("local_anchor_a"));
      definition.localAnchorB = vector(value.at("local_anchor_b"));
      definition.maxForce = float_bits(value, "max_force_bits");
      definition.maxTorque = float_bits(value, "max_torque_bits");
      created = create_typed_joint(definition, declaration);
    } else if (kind == "rope") {
      b2RopeJointDef definition;
      definition.localAnchorA = vector(value.at("local_anchor_a"));
      definition.localAnchorB = vector(value.at("local_anchor_b"));
      definition.maxLength = float_bits(value, "max_length_bits");
      created = create_typed_joint(definition, declaration);
    } else if (kind == "motor") {
      b2MotorJointDef definition;
      definition.linearOffset = vector(value.at("linear_offset"));
      definition.angularOffset = float_bits(value, "angular_offset_bits");
      definition.maxForce = float_bits(value, "max_force_bits");
      definition.maxTorque = float_bits(value, "max_torque_bits");
      definition.correctionFactor = float_bits(value, "correction_factor_bits");
      created = create_typed_joint(definition, declaration);
    }
    if (created == nullptr || !joints_.emplace(id, created).second) {
      throw std::runtime_error("Phase 8 joint identity insertion failed");
    }
    observe_joint(id);
  }

  void mutate_joint(const Json& raw_id, const Json& mutation) {
    auto& target = joint_json(raw_id);
    const auto kind = mutation.at("kind").get<std::string>();
    if (kind == "limit_enabled") {
      if (target.GetType() == e_revoluteJoint) {
        static_cast<b2RevoluteJoint&>(target).EnableLimit(mutation.at("enabled"));
      } else {
        static_cast<b2PrismaticJoint&>(target).EnableLimit(mutation.at("enabled"));
      }
    } else if (kind == "limits") {
      const auto lower = float_bits(mutation, "lower_bits");
      const auto upper = float_bits(mutation, "upper_bits");
      if (target.GetType() == e_revoluteJoint) {
        static_cast<b2RevoluteJoint&>(target).SetLimits(lower, upper);
      } else {
        static_cast<b2PrismaticJoint&>(target).SetLimits(lower, upper);
      }
    } else if (kind == "motor_enabled") {
      const auto enabled = mutation.at("enabled").get<bool>();
      if (target.GetType() == e_revoluteJoint) static_cast<b2RevoluteJoint&>(target).EnableMotor(enabled);
      else if (target.GetType() == e_prismaticJoint) static_cast<b2PrismaticJoint&>(target).EnableMotor(enabled);
      else static_cast<b2WheelJoint&>(target).EnableMotor(enabled);
    } else if (kind == "motor_speed") {
      const auto speed = float_bits(mutation, "speed_bits");
      if (target.GetType() == e_revoluteJoint) static_cast<b2RevoluteJoint&>(target).SetMotorSpeed(speed);
      else if (target.GetType() == e_prismaticJoint) static_cast<b2PrismaticJoint&>(target).SetMotorSpeed(speed);
      else static_cast<b2WheelJoint&>(target).SetMotorSpeed(speed);
    } else if (kind == "max_motor_force") {
      static_cast<b2PrismaticJoint&>(target).SetMaxMotorForce(float_bits(mutation, "force_bits"));
    } else if (kind == "max_motor_torque") {
      const auto torque = float_bits(mutation, "torque_bits");
      if (target.GetType() == e_revoluteJoint) static_cast<b2RevoluteJoint&>(target).SetMaxMotorTorque(torque);
      else static_cast<b2WheelJoint&>(target).SetMaxMotorTorque(torque);
    } else if (kind == "length") static_cast<b2DistanceJoint&>(target).SetLength(float_bits(mutation, "length_bits"));
    else if (kind == "frequency") {
      const auto frequency = float_bits(mutation, "frequency_bits");
      if (target.GetType() == e_distanceJoint) static_cast<b2DistanceJoint&>(target).SetFrequency(frequency);
      else if (target.GetType() == e_mouseJoint) static_cast<b2MouseJoint&>(target).SetFrequency(frequency);
      else if (target.GetType() == e_wheelJoint) static_cast<b2WheelJoint&>(target).SetSpringFrequencyHz(frequency);
      else static_cast<b2WeldJoint&>(target).SetFrequency(frequency);
    } else if (kind == "damping_ratio") {
      const auto ratio = float_bits(mutation, "damping_ratio_bits");
      if (target.GetType() == e_distanceJoint) static_cast<b2DistanceJoint&>(target).SetDampingRatio(ratio);
      else if (target.GetType() == e_mouseJoint) static_cast<b2MouseJoint&>(target).SetDampingRatio(ratio);
      else if (target.GetType() == e_wheelJoint) static_cast<b2WheelJoint&>(target).SetSpringDampingRatio(ratio);
      else static_cast<b2WeldJoint&>(target).SetDampingRatio(ratio);
    } else if (kind == "mouse_target") static_cast<b2MouseJoint&>(target).SetTarget(vector(mutation.at("target")));
    else if (kind == "max_force") {
      const auto force = float_bits(mutation, "force_bits");
      if (target.GetType() == e_mouseJoint) static_cast<b2MouseJoint&>(target).SetMaxForce(force);
      else if (target.GetType() == e_frictionJoint) static_cast<b2FrictionJoint&>(target).SetMaxForce(force);
      else static_cast<b2MotorJoint&>(target).SetMaxForce(force);
    } else if (kind == "max_torque") {
      const auto torque = float_bits(mutation, "torque_bits");
      if (target.GetType() == e_frictionJoint) static_cast<b2FrictionJoint&>(target).SetMaxTorque(torque);
      else static_cast<b2MotorJoint&>(target).SetMaxTorque(torque);
    } else if (kind == "gear_ratio") static_cast<b2GearJoint&>(target).SetRatio(float_bits(mutation, "ratio_bits"));
    else if (kind == "rope_max_length") static_cast<b2RopeJoint&>(target).SetMaxLength(float_bits(mutation, "max_length_bits"));
    else if (kind == "linear_offset") static_cast<b2MotorJoint&>(target).SetLinearOffset(vector(mutation.at("offset")));
    else if (kind == "angular_offset") static_cast<b2MotorJoint&>(target).SetAngularOffset(float_bits(mutation, "offset_bits"));
    else if (kind == "correction_factor") static_cast<b2MotorJoint&>(target).SetCorrectionFactor(float_bits(mutation, "factor_bits"));
    else throw std::runtime_error("unsupported Phase 8 joint mutation");
  }

  std::string branch_state(const b2Joint& value) const {
    if (value.GetType() == e_revoluteJoint) {
      const auto& joint = static_cast<const b2RevoluteJoint&>(value);
      if (!joint.IsLimitEnabled()) return "inactive";
      if (joint.GetLowerLimit() == joint.GetUpperLimit()) return "equal";
      if (joint.GetJointAngle() <= joint.GetLowerLimit()) return "at_lower";
      if (joint.GetJointAngle() >= joint.GetUpperLimit()) return "at_upper";
      return "inactive";
    }
    if (value.GetType() == e_prismaticJoint) {
      const auto& joint = static_cast<const b2PrismaticJoint&>(value);
      if (!joint.IsLimitEnabled()) return "inactive";
      if (joint.GetLowerLimit() == joint.GetUpperLimit()) return "equal";
      if (joint.GetJointTranslation() <= joint.GetLowerLimit()) return "at_lower";
      if (joint.GetJointTranslation() >= joint.GetUpperLimit()) return "at_upper";
      return "inactive";
    }
    if (value.GetType() == e_ropeJoint) {
      const auto& joint = static_cast<const b2RopeJoint&>(value);
      return b2Distance(joint.GetAnchorA(), joint.GetAnchorB()) > joint.GetMaxLength()
                 ? "at_upper"
                 : "inactive";
    }
    if (value.GetType() == e_distanceJoint || value.GetType() == e_pulleyJoint ||
        value.GetType() == e_mouseJoint || value.GetType() == e_frictionJoint) {
      return "inactive";
    }
    return "active";
  }

  float32 coordinate(const b2Joint& value) const {
    if (value.GetType() == e_revoluteJoint) return static_cast<const b2RevoluteJoint&>(value).GetJointAngle();
    if (value.GetType() == e_prismaticJoint) return static_cast<const b2PrismaticJoint&>(value).GetJointTranslation();
    if (value.GetType() == e_distanceJoint || value.GetType() == e_ropeJoint) return b2Distance(value.GetAnchorA(), value.GetAnchorB());
    if (value.GetType() == e_pulleyJoint) {
      const auto& joint = static_cast<const b2PulleyJoint&>(value);
      return joint.GetCurrentLengthA() + joint.GetRatio() * joint.GetCurrentLengthB();
    }
    if (value.GetType() == e_gearJoint) {
      const auto& declaration = joint_declarations_.at(semantic_joint(&value));
      const auto& definition = declaration.at("definition");
      return coordinate(joint_json(definition.at("joint_a_id"))) +
             float_bits(definition, "ratio_bits") *
                 coordinate(joint_json(definition.at("joint_b_id")));
    }
    if (value.GetType() == e_wheelJoint) return static_cast<const b2WheelJoint&>(value).GetJointTranslation();
    if (value.GetType() == e_motorJoint) {
      const auto& joint = static_cast<const b2MotorJoint&>(value);
      auto& mutable_value = const_cast<b2Joint&>(value);
      return mutable_value.GetBodyB()->GetAngle() -
             mutable_value.GetBodyA()->GetAngle() - joint.GetAngularOffset();
    }
    return 0.0F;
  }

  float32 speed(const b2Joint& value) const {
    if (value.GetType() == e_revoluteJoint) return static_cast<const b2RevoluteJoint&>(value).GetJointSpeed();
    if (value.GetType() == e_prismaticJoint) return static_cast<const b2PrismaticJoint&>(value).GetJointSpeed();
    if (value.GetType() == e_wheelJoint) return static_cast<const b2WheelJoint&>(value).GetJointSpeed();
    return 0.0F;
  }

  void observe_joint(const Json& raw_id) {
    const auto id = raw_id.get<std::string>();
    auto& value = joint_by_id(id);
    const auto& declaration = joint_declarations_.at(id);
    Json dependencies = Json::array();
    if (value.GetType() == e_gearJoint) {
      dependencies.push_back(declaration.at("definition").at("joint_a_id"));
      dependencies.push_back(declaration.at("definition").at("joint_b_id"));
    }
    const auto inverse_timestep = 1.0F / float_from_bits(kRigidWorldTimestepBits);
    // Several pinned joint constructors leave solver-direction scratch
    // uninitialized until the first world step. The closed Phase 8 corpus
    // observes these joints before stepping; reading the upstream getter then
    // would be undefined behavior rather than compatibility evidence.
    const auto reaction_force =
        semantic_reaction_force(value, inverse_timestep, solver_initialized_);
    const auto reaction_torque =
        semantic_reaction_torque(value, inverse_timestep, solver_initialized_);
    observations_.push_back(
        {{"kind", "joint"},
         {"snapshot",
          {{"joint_id", id},
           {"joint_kind", joint_kind_name(value.GetType())},
           {"body_a_id", declaration.at("body_a_id")},
           {"body_b_id", declaration.at("body_b_id")},
           {"collide_connected", value.GetCollideConnected()},
           {"dependencies", std::move(dependencies)},
           {"branch_state", branch_state(value)},
           {"coordinate_bits", bits_from_float(coordinate(value))},
           {"speed_bits", bits_from_float(speed(value))},
           {"reaction_force", encode_rigid_vector(reaction_force)},
           {"reaction_torque_bits", bits_from_float(reaction_torque)}}}});
  }

  void create_rope(const Json& raw_id) {
    const auto id = raw_id.get<std::string>();
    const auto& declaration = rope_declarations_.at(id);
    std::vector<b2Vec2> vertices;
    std::vector<float32> masses;
    for (const auto& vertex : declaration.at("vertices")) vertices.push_back(vector(vertex));
    for (const auto& mass : declaration.at("masses_bits")) {
      masses.push_back(float_from_bits(mass.get<std::uint32_t>()));
    }
    b2RopeDef definition;
    definition.vertices = vertices.data();
    definition.count = static_cast<int32>(vertices.size());
    definition.masses = masses.data();
    definition.gravity = vector(declaration.at("gravity"));
    definition.damping = float_bits(declaration, "damping_bits");
    definition.k2 = float_bits(declaration, "stretch_stiffness_bits");
    definition.k3 = float_bits(declaration, "bend_stiffness_bits");
    auto created = std::make_unique<b2Rope>();
    created->Initialize(&definition);
    if (!ropes_.emplace(id, std::move(created)).second) {
      throw std::runtime_error("Phase 8 rope identity insertion failed");
    }
  }

  void observe_rope(const Json& raw_id) {
    const auto id = raw_id.get<std::string>();
    const auto& value = rope_by_id(id);
    Json vertices = Json::array();
    for (int32 index = 0; index < value.GetVertexCount(); ++index) {
      vertices.push_back(encode_rigid_vector(value.GetVertices()[index]));
    }
    observations_.push_back(
        {{"kind", "rope"},
         {"snapshot", {{"rope_id", id}, {"vertices", std::move(vertices)}}}});
  }

  void reconstruct() {
    std::uint32_t ordinal = 0;
    for (auto body_it = timeline_.at("bodies").rbegin(); body_it != timeline_.at("bodies").rend(); ++body_it) {
      const auto id = body_it->at("body_id").get<std::string>();
      if (!bodies_.count(id)) continue;
      observations_.push_back(reconstruction(ordinal++, "body", id));
      for (auto fixture_it = timeline_.at("fixtures").rbegin(); fixture_it != timeline_.at("fixtures").rend(); ++fixture_it) {
        if (fixture_it->at("owner_body_id") == id && fixtures_.count(fixture_it->at("fixture_id").get<std::string>())) {
          observations_.push_back(reconstruction(ordinal++, "fixture", fixture_it->at("fixture_id").get<std::string>()));
        }
      }
    }
    const auto joint_records = timeline_.value("joints", Json::array());
    for (auto joint_it = joint_records.rbegin(); joint_it != joint_records.rend(); ++joint_it) {
      const auto id = joint_it->at("joint_id").get<std::string>();
      if (!joints_.count(id) || joint_it->at("definition").at("kind") == "gear") continue;
      auto record = reconstruction(ordinal++, "joint", id);
      if (joint_it->at("definition").at("kind") == "mouse") {
        record["record"]["support"] = "unsupported_mouse_joint";
      }
      observations_.push_back(std::move(record));
    }
    for (auto joint_it = joint_records.rbegin(); joint_it != joint_records.rend(); ++joint_it) {
      const auto id = joint_it->at("joint_id").get<std::string>();
      if (!joints_.count(id) || joint_it->at("definition").at("kind") != "gear") continue;
      auto record = reconstruction(ordinal++, "joint", id);
      record["record"]["dependency_ids"] = {
          joint_it->at("definition").at("joint_a_id"),
          joint_it->at("definition").at("joint_b_id")};
      observations_.push_back(std::move(record));
    }
  }

  static Json reconstruction(std::uint32_t ordinal, std::string_view kind, const std::string& id) {
    return {
        {"kind", "reconstruction"},
        {"record",
         {{"ordinal", ordinal},
          {"kind", kind},
          {"entity_id", id},
          {"support", "supported"},
          {"dependency_ids", Json::array()}}}};
  }

  void diagnostics() {
    observations_.push_back(
        {{"kind", "diagnostics"},
         {"snapshot",
          {{"body_count", static_cast<std::uint32_t>(bodies_.size())},
           {"fixture_count", static_cast<std::uint32_t>(fixtures_.size())},
           {"joint_count", static_cast<std::uint32_t>(joints_.size())},
           {"contact_count", static_cast<std::uint32_t>(world_.GetContactCount())},
           {"tree_height", static_cast<std::uint32_t>(world_.GetTreeHeight())},
           {"tree_max_balance", static_cast<std::uint32_t>(world_.GetTreeBalance())},
           {"tree_quality_bits", bits_from_float(world_.GetTreeQuality())}}}});
  }

  void destroy_joint(const Json& raw_id) {
    const auto id = raw_id.get<std::string>();
    auto found = joints_.find(id);
    if (found == joints_.end()) throw std::runtime_error("Phase 8 joint is not live");
    const auto declaration = joint_declarations_.at(id);
    std::vector<std::string> dependent_gears;
    for (const auto& [candidate_id, candidate] : joints_) {
      if (candidate->GetType() != e_gearJoint) continue;
      const auto& definition = joint_declarations_.at(candidate_id).at("definition");
      if (definition.at("joint_a_id") == id || definition.at("joint_b_id") == id) {
        dependent_gears.push_back(candidate_id);
      }
    }
    for (const auto& dependent_id : dependent_gears) {
      const auto dependent = joints_.find(dependent_id);
      if (dependent == joints_.end()) continue;
      push_entity_lifecycle("joint_goodbye", dependent_id);
      world_.DestroyJoint(dependent->second);
      joints_.erase(dependent);
    }
    world_.DestroyJoint(found->second);
    joints_.erase(found);
    if (!declaration.at("collide_connected").get<bool>()) {
      refilter_body_pair(
          declaration.at("body_a_id").get<std::string>(),
          declaration.at("body_b_id").get<std::string>());
    }
  }

  void refilter_body_pair(
      const std::string& body_a_id,
      const std::string& body_b_id) {
    for (const auto& [fixture_id, value] : fixtures_) {
      const auto owner = fixture_declarations_.at(fixture_id)
                             .at("owner_body_id")
                             .get<std::string>();
      if (owner == body_a_id || owner == body_b_id) {
        value->SetFilterData(value->GetFilterData());
      }
    }
  }

  void destroy_fixture(const Json& raw_id) {
    const auto id = raw_id.get<std::string>();
    auto found = fixtures_.find(id);
    if (found == fixtures_.end()) throw std::runtime_error("Phase 8 fixture is not live");
    destroying_fixture_or_body_ = true;
    found->second->GetBody()->DestroyFixture(found->second);
    destroying_fixture_or_body_ = false;
    fixtures_.erase(found);
  }

  void destroy_body(const Json& raw_id) {
    const auto id = raw_id.get<std::string>();
    auto found = bodies_.find(id);
    if (found == bodies_.end()) throw std::runtime_error("Phase 8 body is not live");
    auto* target = found->second;
    destroying_fixture_or_body_ = true;
    world_.DestroyBody(target);
    destroying_fixture_or_body_ = false;
    bodies_.erase(found);
    push_entity_lifecycle("body_destroyed", id);
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

  Json fixture_snapshots() const {
    Json result = Json::array();
    for (const auto& declaration : timeline_.at("fixtures")) {
      const auto id = declaration.at("fixture_id").get<std::string>();
      const auto found = fixtures_.find(id);
      if (found == fixtures_.end()) continue;
      const auto& value = *found->second;
      result.push_back(
          {{"fixture_id", id},
           {"owner_body_id", declaration.at("owner_body_id")},
           {"sensor", value.IsSensor()},
           {"density_bits", bits_from_float(value.GetDensity())},
           {"friction_bits", bits_from_float(value.GetFriction())},
           {"restitution_bits", bits_from_float(value.GetRestitution())},
           {"filter", encode_rigid_filter(value.GetFilterData())}});
    }
    return result;
  }

  static std::string_view feature_kind(std::uint8_t type) {
    return type == b2ContactFeature::e_vertex ? "vertex" : "face";
  }

  static Json manifold_json(const b2Manifold& manifold) {
    Json points = Json::array();
    for (int32 index = 0; index < manifold.pointCount; ++index) {
      const auto& point = manifold.points[index];
      points.push_back(
          {{"point", encode_rigid_vector(point.localPoint)},
           {"feature",
            {{"index_a", point.id.cf.indexA},
             {"index_b", point.id.cf.indexB},
             {"kind_a", feature_kind(point.id.cf.typeA)},
             {"kind_b", feature_kind(point.id.cf.typeB)}}},
           {"normal_impulse_bits", bits_from_float(point.normalImpulse)},
           {"tangent_impulse_bits", bits_from_float(point.tangentImpulse)}});
    }
    const auto kind = manifold.type == b2Manifold::e_circles
                          ? "circles"
                          : manifold.type == b2Manifold::e_faceA ? "face_a"
                                                                 : "face_b";
    return {
        {"manifold_kind", kind},
        {"local_normal", encode_rigid_vector(manifold.localNormal)},
        {"local_point", encode_rigid_vector(manifold.localPoint)},
        {"points", std::move(points)}};
  }

  Json contact_snapshots() {
    Json result = Json::array();
    for (auto* contact = world_.GetContactList(); contact != nullptr;
         contact = contact->GetNext()) {
      const auto sensor = contact->GetFixtureA()->IsSensor() ||
                          contact->GetFixtureB()->IsSensor();
      Json maybe_manifold = nullptr;
      if (!sensor && contact->GetManifold()->pointCount > 0) {
        maybe_manifold = manifold_json(*contact->GetManifold());
      }
      result.push_back(
          {{"identity", contact_identity(contact)},
           {"touching", contact->IsTouching()},
           {"enabled", contact->IsEnabled()},
           {"sensor", sensor},
           {"mixed_friction_bits", bits_from_float(contact->GetFriction())},
           {"mixed_restitution_bits", bits_from_float(contact->GetRestitution())},
           {"maybe_manifold", std::move(maybe_manifold)}});
    }
    return result;
  }

  Json capture(const Json& checkpoint) {
    if (observations_.size() > kMaximumPhase8Observations) {
      throw std::runtime_error("Phase 8 observation count outside reviewed bounds");
    }
    auto bodies = body_snapshots();
    auto fixtures = fixture_snapshots();
    auto contacts = contact_snapshots();
    std::uint32_t manifold_points = 0;
    for (const auto& contact : contacts) {
      if (!contact.at("maybe_manifold").is_null()) {
        manifold_points += static_cast<std::uint32_t>(
            contact.at("maybe_manifold").at("points").size());
      }
    }
    const Json actual_counts{
        {"bodies", static_cast<std::uint32_t>(bodies.size())},
        {"fixtures", static_cast<std::uint32_t>(fixtures.size())},
        {"contacts", static_cast<std::uint32_t>(contacts.size())},
        {"manifold_points", manifold_points},
        {"events", 0U},
        {"destructions", 0U}};
    if (actual_counts != checkpoint.at("counts")) {
      throw std::runtime_error(
          "Phase 8 checkpoint count mismatch at " +
          checkpoint.at("checkpoint_id").get<std::string>() + ": actual=" +
          actual_counts.dump() + ", expected=" + checkpoint.at("counts").dump());
    }
    Json result{
        {"checkpoint_id", checkpoint.at("checkpoint_id")},
        {"phase", checkpoint.at("phase")},
        {"counts", std::move(actual_counts)},
        {"bodies", std::move(bodies)},
        {"fixtures", std::move(fixtures)},
        {"contacts", std::move(contacts)},
        {"events", Json::array()},
        {"destructions", Json::array()}};
    if (!observations_.empty()) result["observations"] = std::move(observations_);
    observations_ = Json::array();
    return result;
  }

  b2Body& body(const Json& raw_id) { return body(raw_id.get<std::string>()); }
  b2Body& body(const std::string& id) {
    const auto found = bodies_.find(id);
    if (found == bodies_.end()) throw std::runtime_error("Phase 8 body is not live");
    return *found->second;
  }
  b2Fixture& fixture(const Json& raw_id) {
    const auto found = fixtures_.find(raw_id.get<std::string>());
    if (found == fixtures_.end()) throw std::runtime_error("Phase 8 fixture is not live");
    return *found->second;
  }
  b2Joint& joint_json(const Json& raw_id) const {
    return joint_by_id(raw_id.get<std::string>());
  }
  b2Joint& joint_by_id(const std::string& id) const {
    const auto found = joints_.find(id);
    if (found == joints_.end()) throw std::runtime_error("Phase 8 joint is not live");
    return *found->second;
  }
  b2Rope& rope_json(const Json& raw_id) const {
    return rope_by_id(raw_id.get<std::string>());
  }
  b2Rope& rope_by_id(const std::string& id) const {
    const auto found = ropes_.find(id);
    if (found == ropes_.end()) throw std::runtime_error("Phase 8 rope is not live");
    return *found->second;
  }
  std::string semantic_joint(const b2Joint* joint_value) const {
    const auto found = std::find_if(joints_.begin(), joints_.end(), [&](const auto& item) {
      return item.second == joint_value;
    });
    if (found == joints_.end()) throw std::runtime_error("Phase 8 joint identity is unmapped");
    return found->first;
  }

  std::string maybe_semantic_joint(const b2Joint* joint_value) const {
    const auto found = std::find_if(joints_.begin(), joints_.end(), [&](const auto& item) {
      return item.second == joint_value;
    });
    return found == joints_.end() ? std::string{} : found->first;
  }

  std::string semantic_fixture(const b2Fixture* fixture_value) const {
    const auto id = maybe_semantic_fixture(fixture_value);
    if (id.empty()) throw std::runtime_error("Phase 8 fixture identity is unmapped");
    return id;
  }

  std::string maybe_semantic_fixture(const b2Fixture* fixture_value) const {
    const auto found = std::find_if(fixtures_.begin(), fixtures_.end(), [&](const auto& item) {
      return item.second == fixture_value;
    });
    return found == fixtures_.end() ? std::string{} : found->first;
  }

  Json contact_identity(const b2Contact* contact) {
    auto fixture_a_id = semantic_fixture(contact->GetFixtureA());
    auto fixture_b_id = semantic_fixture(contact->GetFixtureB());
    auto child_a = static_cast<std::uint32_t>(contact->GetChildIndexA());
    auto child_b = static_cast<std::uint32_t>(contact->GetChildIndexB());
    if (fixture_order(fixture_b_id) < fixture_order(fixture_a_id)) {
      std::swap(fixture_a_id, fixture_b_id);
      std::swap(child_a, child_b);
    }
    return {
        {"fixture_a_id", fixture_a_id},
        {"child_a", child_a},
        {"fixture_b_id", fixture_b_id},
        {"child_b", child_b},
        {"occurrence", 1}};
  }

  std::size_t fixture_order(const std::string& id) const {
    const auto& declarations = timeline_.at("fixtures");
    const auto found = std::find_if(declarations.begin(), declarations.end(), [&](const auto& item) {
      return item.at("fixture_id") == id;
    });
    if (found == declarations.end()) {
      throw std::runtime_error("Phase 8 fixture declaration is unmapped");
    }
    return static_cast<std::size_t>(std::distance(declarations.begin(), found));
  }

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
