#include "catalog_joint.hpp"

#include <stdexcept>
#include <string>

namespace liquidfun::reference::catalog_run_detail {
namespace {

enum class CatalogJointKind {
  revolute,
  prismatic,
  distance,
  pulley,
  mouse,
  gear,
  wheel,
  weld,
  friction,
  rope,
  motor,
};

CatalogJointKind joint_kind(std::string_view slug) {
  if (slug == "joint-revolute-behavior") return CatalogJointKind::revolute;
  if (slug == "joint-prismatic-behavior") return CatalogJointKind::prismatic;
  if (slug == "joint-distance-behavior") return CatalogJointKind::distance;
  if (slug == "joint-pulley-behavior") return CatalogJointKind::pulley;
  if (slug == "joint-mouse-behavior") return CatalogJointKind::mouse;
  if (slug == "joint-gear-behavior") return CatalogJointKind::gear;
  if (slug == "joint-wheel-behavior") return CatalogJointKind::wheel;
  if (slug == "joint-weld-behavior") return CatalogJointKind::weld;
  if (slug == "joint-friction-behavior") return CatalogJointKind::friction;
  if (slug == "joint-rope-behavior") return CatalogJointKind::rope;
  if (slug == "joint-motor-behavior") return CatalogJointKind::motor;
  throw std::runtime_error("catalog scenario does not declare a joint kind");
}

template <typename Definition>
b2Joint* create_typed_joint(
    Definition& definition,
    b2Body* body_a,
    b2Body* body_b,
    b2World& world) {
  definition.bodyA = body_a;
  definition.bodyB = body_b;
  auto* joint = world.CreateJoint(&definition);
  if (joint == nullptr) {
    throw std::runtime_error("pinned world failed to create catalog joint");
  }
  return joint;
}

template <typename Joint>
Joint& require_joint(
    b2Joint& joint,
    b2JointType expected,
    std::string_view mutation) {
  if (joint.GetType() != expected) {
    throw std::runtime_error(
        std::string(mutation) + " mutation does not match catalog joint kind");
  }
  return static_cast<Joint&>(joint);
}

b2Joint* create_gear_joint(
    const std::vector<b2Body*>& bodies,
    const std::vector<b2Joint*>& existing_joints,
    b2World& world) {
  if (existing_joints.size() < 2U) {
    b2RevoluteJointDef definition;
    const auto offset = existing_joints.size() * 2U;
    return create_typed_joint(
        definition, bodies.at(offset), bodies.at(offset + 1U), world);
  }
  if (existing_joints.size() != 2U) {
    throw std::runtime_error("catalog gear topology is invalid");
  }
  b2GearJointDef definition;
  definition.joint1 = existing_joints.at(0);
  definition.joint2 = existing_joints.at(1);
  definition.ratio = 1.0F;
  return create_typed_joint(
      definition, bodies.at(1), bodies.at(3), world);
}

}  // namespace

b2Joint* create_catalog_joint(
    std::string_view slug,
    const std::vector<b2Body*>& bodies,
    const std::vector<b2Joint*>& existing_joints,
    b2World& world) {
  const auto kind = joint_kind(slug);
  if (kind == CatalogJointKind::gear) {
    if (bodies.size() < 4U) {
      throw std::runtime_error("catalog gear requires four bodies");
    }
    return create_gear_joint(bodies, existing_joints, world);
  }
  if (bodies.size() < 2U || !existing_joints.empty()) {
    throw std::runtime_error("catalog joint topology is invalid");
  }
  auto* body_a = bodies.at(0);
  auto* body_b = bodies.at(1);
  switch (kind) {
    case CatalogJointKind::revolute: {
      b2RevoluteJointDef definition;
      return create_typed_joint(definition, body_a, body_b, world);
    }
    case CatalogJointKind::prismatic: {
      b2PrismaticJointDef definition;
      definition.localAxisA.Set(1.0F, 0.0F);
      return create_typed_joint(definition, body_a, body_b, world);
    }
    case CatalogJointKind::distance: {
      b2DistanceJointDef definition;
      definition.length = 1.0F;
      return create_typed_joint(definition, body_a, body_b, world);
    }
    case CatalogJointKind::pulley: {
      b2PulleyJointDef definition;
      definition.groundAnchorA.Set(-1.0F, 2.0F);
      definition.groundAnchorB.Set(1.0F, 2.0F);
      definition.lengthA = 2.0F;
      definition.lengthB = 2.0F;
      definition.ratio = 1.0F;
      return create_typed_joint(definition, body_a, body_b, world);
    }
    case CatalogJointKind::mouse: {
      b2MouseJointDef definition;
      definition.target = body_b->GetPosition();
      definition.maxForce = 10.0F;
      return create_typed_joint(definition, body_a, body_b, world);
    }
    case CatalogJointKind::wheel: {
      b2WheelJointDef definition;
      definition.localAxisA.Set(1.0F, 0.0F);
      return create_typed_joint(definition, body_a, body_b, world);
    }
    case CatalogJointKind::weld: {
      b2WeldJointDef definition;
      return create_typed_joint(definition, body_a, body_b, world);
    }
    case CatalogJointKind::friction: {
      b2FrictionJointDef definition;
      definition.maxForce = 1.0F;
      definition.maxTorque = 1.0F;
      return create_typed_joint(definition, body_a, body_b, world);
    }
    case CatalogJointKind::rope: {
      b2RopeJointDef definition;
      definition.maxLength = 2.0F;
      return create_typed_joint(definition, body_a, body_b, world);
    }
    case CatalogJointKind::motor: {
      b2MotorJointDef definition;
      definition.Initialize(body_a, body_b);
      return create_typed_joint(definition, body_a, body_b, world);
    }
    case CatalogJointKind::gear:
      break;
  }
  throw std::runtime_error("unsupported catalog joint kind");
}

void mutate_catalog_joint(b2Joint& joint, const Json& mutation) {
  const auto kind = as_id(mutation.at("kind"), "joint mutation kind");
  if (kind == "limit_enabled") {
    const auto enabled = mutation.at("enabled").get<bool>();
    if (joint.GetType() == e_revoluteJoint) {
      static_cast<b2RevoluteJoint&>(joint).EnableLimit(enabled);
    } else {
      require_joint<b2PrismaticJoint>(joint, e_prismaticJoint, kind)
          .EnableLimit(enabled);
    }
  } else if (kind == "limits") {
    const auto lower = as_finite_float(mutation.at("lower_bits"), "lower limit");
    const auto upper = as_finite_float(mutation.at("upper_bits"), "upper limit");
    if (joint.GetType() == e_revoluteJoint) {
      static_cast<b2RevoluteJoint&>(joint).SetLimits(lower, upper);
    } else {
      require_joint<b2PrismaticJoint>(joint, e_prismaticJoint, kind)
          .SetLimits(lower, upper);
    }
  } else if (kind == "motor_enabled") {
    const auto enabled = mutation.at("enabled").get<bool>();
    if (joint.GetType() == e_revoluteJoint) {
      static_cast<b2RevoluteJoint&>(joint).EnableMotor(enabled);
    } else if (joint.GetType() == e_prismaticJoint) {
      static_cast<b2PrismaticJoint&>(joint).EnableMotor(enabled);
    } else {
      require_joint<b2WheelJoint>(joint, e_wheelJoint, kind)
          .EnableMotor(enabled);
    }
  } else if (kind == "motor_speed") {
    const auto speed = as_finite_float(mutation.at("speed_bits"), "motor speed");
    if (joint.GetType() == e_revoluteJoint) {
      static_cast<b2RevoluteJoint&>(joint).SetMotorSpeed(speed);
    } else if (joint.GetType() == e_prismaticJoint) {
      static_cast<b2PrismaticJoint&>(joint).SetMotorSpeed(speed);
    } else {
      require_joint<b2WheelJoint>(joint, e_wheelJoint, kind)
          .SetMotorSpeed(speed);
    }
  } else if (kind == "max_motor_force") {
    require_joint<b2PrismaticJoint>(joint, e_prismaticJoint, kind)
        .SetMaxMotorForce(
            as_finite_float(mutation.at("force_bits"), "motor force"));
  } else if (kind == "max_motor_torque") {
    const auto torque =
        as_finite_float(mutation.at("torque_bits"), "motor torque");
    if (joint.GetType() == e_revoluteJoint) {
      static_cast<b2RevoluteJoint&>(joint).SetMaxMotorTorque(torque);
    } else {
      require_joint<b2WheelJoint>(joint, e_wheelJoint, kind)
          .SetMaxMotorTorque(torque);
    }
  } else if (kind == "length") {
    require_joint<b2DistanceJoint>(joint, e_distanceJoint, kind)
        .SetLength(as_finite_float(mutation.at("length_bits"), "length"));
  } else if (kind == "frequency") {
    const auto frequency =
        as_finite_float(mutation.at("frequency_bits"), "frequency");
    if (joint.GetType() == e_distanceJoint) {
      static_cast<b2DistanceJoint&>(joint).SetFrequency(frequency);
    } else if (joint.GetType() == e_mouseJoint) {
      static_cast<b2MouseJoint&>(joint).SetFrequency(frequency);
    } else if (joint.GetType() == e_wheelJoint) {
      static_cast<b2WheelJoint&>(joint).SetSpringFrequencyHz(frequency);
    } else {
      require_joint<b2WeldJoint>(joint, e_weldJoint, kind)
          .SetFrequency(frequency);
    }
  } else if (kind == "damping_ratio") {
    const auto ratio =
        as_finite_float(mutation.at("damping_ratio_bits"), "damping ratio");
    if (joint.GetType() == e_distanceJoint) {
      static_cast<b2DistanceJoint&>(joint).SetDampingRatio(ratio);
    } else if (joint.GetType() == e_mouseJoint) {
      static_cast<b2MouseJoint&>(joint).SetDampingRatio(ratio);
    } else if (joint.GetType() == e_wheelJoint) {
      static_cast<b2WheelJoint&>(joint).SetSpringDampingRatio(ratio);
    } else {
      require_joint<b2WeldJoint>(joint, e_weldJoint, kind)
          .SetDampingRatio(ratio);
    }
  } else if (kind == "mouse_target") {
    require_joint<b2MouseJoint>(joint, e_mouseJoint, kind)
        .SetTarget(as_vec2(mutation.at("target"), "mouse target"));
  } else if (kind == "max_force") {
    const auto force = as_finite_float(mutation.at("force_bits"), "max force");
    if (joint.GetType() == e_mouseJoint) {
      static_cast<b2MouseJoint&>(joint).SetMaxForce(force);
    } else if (joint.GetType() == e_frictionJoint) {
      static_cast<b2FrictionJoint&>(joint).SetMaxForce(force);
    } else {
      require_joint<b2MotorJoint>(joint, e_motorJoint, kind).SetMaxForce(force);
    }
  } else if (kind == "max_torque") {
    const auto torque =
        as_finite_float(mutation.at("torque_bits"), "max torque");
    if (joint.GetType() == e_frictionJoint) {
      static_cast<b2FrictionJoint&>(joint).SetMaxTorque(torque);
    } else {
      require_joint<b2MotorJoint>(joint, e_motorJoint, kind)
          .SetMaxTorque(torque);
    }
  } else if (kind == "gear_ratio") {
    require_joint<b2GearJoint>(joint, e_gearJoint, kind)
        .SetRatio(as_finite_float(mutation.at("ratio_bits"), "gear ratio"));
  } else if (kind == "rope_max_length") {
    require_joint<b2RopeJoint>(joint, e_ropeJoint, kind)
        .SetMaxLength(
            as_finite_float(mutation.at("max_length_bits"), "rope max length"));
  } else if (kind == "linear_offset") {
    require_joint<b2MotorJoint>(joint, e_motorJoint, kind)
        .SetLinearOffset(as_vec2(mutation.at("offset"), "linear offset"));
  } else if (kind == "angular_offset") {
    require_joint<b2MotorJoint>(joint, e_motorJoint, kind)
        .SetAngularOffset(
            as_finite_float(mutation.at("offset_bits"), "angular offset"));
  } else if (kind == "correction_factor") {
    require_joint<b2MotorJoint>(joint, e_motorJoint, kind)
        .SetCorrectionFactor(
            as_finite_float(mutation.at("factor_bits"), "correction factor"));
  } else {
    throw std::runtime_error("unknown joint mutation");
  }
}

}  // namespace liquidfun::reference::catalog_run_detail
