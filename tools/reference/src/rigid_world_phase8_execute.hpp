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
#include <utility>
#include <vector>

namespace liquidfun::reference {
namespace phase8_detail {

using Json = nlohmann::json;

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
    for (const auto& joint : timeline_.value("joints", Json::array())) {
      joint_declarations_.emplace(joint.at("joint_id").get<std::string>(), joint);
    }
    for (const auto& rope : timeline_.value("ropes", Json::array())) {
      rope_declarations_.emplace(rope.at("rope_id").get<std::string>(), rope);
    }
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
      return;
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
    if (kind == "set_contact_filter_directive" ||
        kind == "set_pre_solve_directive") {
      return;
    }
    if (kind == "destroy_fixture") return destroy_fixture(action.at("fixture_id"));
    if (kind == "destroy_body") return destroy_body(action.at("body_id"));
    throw std::runtime_error("unsupported Phase 8 execution action");
  }

  static float32 float_bits(const Json& value, std::string_view name) {
    return float_from_bits(value.at(name).get<std::uint32_t>());
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
    observe_rope(id);
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
      observations_.push_back(reconstruction(ordinal++, "joint", id));
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
    world_.DestroyJoint(found->second);
    joints_.erase(found);
  }

  void destroy_fixture(const Json& raw_id) {
    const auto id = raw_id.get<std::string>();
    auto found = fixtures_.find(id);
    if (found == fixtures_.end()) throw std::runtime_error("Phase 8 fixture is not live");
    found->second->GetBody()->DestroyFixture(found->second);
    fixtures_.erase(found);
  }

  void destroy_body(const Json& raw_id) {
    const auto id = raw_id.get<std::string>();
    auto found = bodies_.find(id);
    if (found == bodies_.end()) throw std::runtime_error("Phase 8 body is not live");
    auto* target = found->second;
    for (auto fixture_it = fixtures_.begin(); fixture_it != fixtures_.end();) {
      fixture_it = fixture_it->second->GetBody() == target ? fixtures_.erase(fixture_it) : std::next(fixture_it);
    }
    world_.DestroyBody(target);
    bodies_.erase(found);
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

  Json capture(const Json& checkpoint) {
    auto bodies = body_snapshots();
    auto fixtures = fixture_snapshots();
    Json result{
        {"checkpoint_id", checkpoint.at("checkpoint_id")},
        {"phase", checkpoint.at("phase")},
        {"counts", checkpoint.at("counts")},
        {"bodies", std::move(bodies)},
        {"fixtures", std::move(fixtures)},
        {"contacts", Json::array()},
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
  Json observations_ = Json::array();
  bool solver_initialized_ = false;
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
