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
