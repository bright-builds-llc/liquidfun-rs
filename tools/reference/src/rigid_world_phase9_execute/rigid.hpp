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
    if (kind == "set_body_type") {
      body(action.at("body_id"))
          .SetType(body_type(action.at("body_kind").get<std::string>()));
      return;
    }
    if (kind == "set_body_transform") {
      body(action.at("body_id"))
          .SetTransform(
              phase9_vector(action.at("transform").at("position")),
              float_from_bits(
                  action.at("transform").at("angle_bits").get<std::uint32_t>()));
      return;
    }
    if (kind == "set_body_active") {
      body(action.at("body_id")).SetActive(action.at("active").get<bool>());
      return;
    }
    if (kind == "set_fixture_sensor") {
      fixture(action.at("fixture_id")).SetSensor(action.at("sensor").get<bool>());
      return;
    }
    if (kind == "set_fixture_material") {
      auto& target = fixture(action.at("fixture_id"));
      target.SetFriction(
          float_from_bits(action.at("friction_bits").get<std::uint32_t>()));
      target.SetRestitution(
          float_from_bits(action.at("restitution_bits").get<std::uint32_t>()));
      return;
    }
    if (kind == "set_fixture_filter") {
      const auto& raw = action.at("filter");
      b2Filter filter;
      filter.categoryBits = raw.at("category_bits").get<std::uint16_t>();
      filter.maskBits = raw.at("mask_bits").get<std::uint16_t>();
      filter.groupIndex = raw.at("group_index").get<std::int16_t>();
      fixture(action.at("fixture_id")).SetFilterData(filter);
      return;
    }
    if (kind == "set_fixture_density") {
      fixture(action.at("fixture_id"))
          .SetDensity(
              float_from_bits(action.at("density_bits").get<std::uint32_t>()));
      return;
    }
    if (kind == "reset_mass_data") {
      body(action.at("body_id")).ResetMassData();
      return;
    }
    if (kind == "set_custom_mass_data") {
      b2MassData data;
      data.mass = float_from_bits(action.at("mass_bits").get<std::uint32_t>());
      data.center = phase9_vector(action.at("center"));
      data.I = float_from_bits(action.at("inertia_bits").get<std::uint32_t>());
      body(action.at("body_id")).SetMassData(&data);
      return;
    }
    if (kind == "inspect_body" || kind == "inspect_fixture") {
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
      return;
    }
    throw std::runtime_error("unsupported retained rigid action in Phase 9 execution");
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

  void observe_mixed_state() {
    observations_.push_back(
        {{"kind", "particle"},
         {"observation",
          {{"kind", "mixed_state"},
           {"body_ids", semantic_body_ids()},
           {"particle_ids", semantic_particle_ids()}}}});
  }
