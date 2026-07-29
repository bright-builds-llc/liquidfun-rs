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
      const auto& gear = static_cast<const b2GearJoint&>(value);
      const auto& definition = joint_declarations_.at(semantic_joint(&value)).at("definition");
      return coordinate(joint_json(definition.at("joint_a_id"))) +
             gear.GetRatio() *
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
