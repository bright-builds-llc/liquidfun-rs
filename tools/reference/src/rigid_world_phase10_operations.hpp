// TimelineExecution operation methods. Included inside the private class.

  void execute(const Json& action) {
    const auto kind = action.at("kind").get<std::string>();
    if (kind == "particle") return execute_particle(action.at("action"));
    if (kind == "particle_group") return execute_group(action.at("operation"));
    if (kind == "create_body") return create_body(action);
    if (kind == "create_fixture") return create_fixture(action);
    if (kind == "destroy_fixture") return destroy_fixture(action);
    if (kind == "destroy_body") return destroy_body(action);
    // Retained rigid actions are executed by the established rigid adapter.
    // A Phase 10 overlay needs only live collision geometry and group state.
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
    definition.radius =
        float_from_bits(raw.at("radius_bits").get<std::uint32_t>());
    definition.dampingStrength =
        float_from_bits(raw.at("damping_bits").get<std::uint32_t>());
    definition.destroyByAge = raw.at("destruction_by_age").get<bool>();
    definition.lifetimeGranularity = float_from_bits(
        raw.at("lifetime_granularity_bits").get<std::uint32_t>());
    if (!raw.at("maximum_count").is_null()) {
      definition.maxCount = raw.at("maximum_count").get<int32>();
    }
    auto state = std::make_unique<SystemState>();
    state->system = world_.CreateParticleSystem(&definition);
    if (state->system == nullptr) {
      throw std::runtime_error("pinned world failed to create Phase 10 system");
    }
    const auto& buffer = raw.at("buffer_mode");
    state->fixed = buffer.at("kind") == "fixed";
    state->declared_capacity = state->fixed
                                   ? buffer.at("capacity").get<std::size_t>()
                                   : raw.at("maximum_count").is_null()
                                         ? std::size_t{512}
                                         : raw.at("maximum_count").get<std::size_t>();
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
      throw std::runtime_error("duplicate live Phase 10 system");
    }
  }

  void execute_particle(const Json& action) {
    const auto kind = action.at("kind").get<std::string>();
    if (kind == "create_system") {
      create_system(action);
      Json body_ids = Json::array();
      for (const auto& declaration : timeline_.at("bodies")) {
        const auto id = declaration.at("body_id").get<std::string>();
        if (bodies_.count(id) != 0U) body_ids.push_back(id);
      }
      observations_.push_back(
          {{"kind", "particle"},
           {"observation",
            {{"kind", "mixed_state"},
             {"body_ids", std::move(body_ids)},
             {"particle_ids", Json::array()}}}});
      return;
    }
    if (kind == "destroy_system") {
      const auto id = action.at("system_id").get<std::string>();
      auto found = systems_.find(id);
      if (found == systems_.end()) {
        throw std::runtime_error("Phase 10 system is not live");
      }
      auto* doomed = found->second->system;
      world_.DestroyParticleSystem(doomed);
      systems_.erase(found);
      particles_.erase(
          std::remove_if(
              particles_.begin(), particles_.end(),
              [&](const auto& binding) { return binding.system == doomed; }),
          particles_.end());
      groups_.erase(
          std::remove_if(
              groups_.begin(), groups_.end(),
              [&](const auto& binding) { return binding.system_id == id; }),
          groups_.end());
      observations_.push_back(
          {{"kind", "particle"},
           {"observation",
            {{"kind", "lifecycle"},
             {"occurrence",
              {{"ordinal", next_phase9_occurrence_ordinal_++},
               {"kind", "system_destroyed"},
               {"system_id", id},
               {"maybe_particle_id", nullptr},
               {"maybe_other_particle_id", nullptr},
               {"maybe_fixture_id", nullptr}}}}}});
      return;
    }
    // Phase 10 scenarios create members through group definitions. Other
    // inherited Phase 9 actions are intentionally left to the Phase 9 overlay.
  }

  std::unique_ptr<b2Shape> shape(const Json& raw) {
    const auto kind = raw.at("kind").get<std::string>();
    if (kind == "circle") {
      auto result = std::make_unique<b2CircleShape>();
      result->m_p = phase10_vector(raw.at("center"));
      result->m_radius =
          float_from_bits(raw.at("radius_bits").get<std::uint32_t>());
      return result;
    }
    if (kind == "polygon") {
      auto result = std::make_unique<b2PolygonShape>();
      std::vector<b2Vec2> vertices;
      for (const auto& vertex : raw.at("vertices")) {
        vertices.push_back(phase10_vector(vertex));
      }
      result->Set(vertices.data(), static_cast<int32>(vertices.size()));
      return result;
    }
    if (kind == "edge") {
      auto result = std::make_unique<b2EdgeShape>();
      result->Set(
          phase10_vector(raw.at("vertex_a")),
          phase10_vector(raw.at("vertex_b")));
      return result;
    }
    auto result = std::make_unique<b2ChainShape>();
    std::vector<b2Vec2> vertices;
    for (const auto& vertex : raw.at("vertices")) {
      vertices.push_back(phase10_vector(vertex));
    }
    if (raw.at("looped").get<bool>()) {
      result->CreateLoop(vertices.data(), static_cast<int32>(vertices.size()));
    } else {
      result->CreateChain(vertices.data(), static_cast<int32>(vertices.size()));
    }
    return result;
  }

  void create_group(const Json& definition) {
    auto& owner = system(definition.at("system_id"));
    const auto before_count = owner.system->GetParticleCount();
    std::vector<const b2ParticleHandle*> prior_handles;
    prior_handles.reserve(static_cast<std::size_t>(before_count));
    for (int32 index = 0; index < before_count; ++index) {
      prior_handles.push_back(owner.system->GetParticleHandleFromIndex(index));
    }
    b2ParticleGroupDef group_definition;
    group_definition.flags = definition.at("particle_flags_bits").get<uint32>();
    group_definition.groupFlags = definition.at("group_flags_bits").get<uint32>();
    group_definition.position =
        phase10_vector(definition.at("transform").at("position"));
    group_definition.angle = float_from_bits(
        definition.at("transform").at("angle_bits").get<std::uint32_t>());
    group_definition.linearVelocity = phase10_vector(definition.at("linear_velocity"));
    group_definition.angularVelocity = float_from_bits(
        definition.at("angular_velocity_bits").get<std::uint32_t>());
    const auto& color = definition.at("color");
    group_definition.color = b2ParticleColor(
        color.at(0).get<uint8>(), color.at(1).get<uint8>(),
        color.at(2).get<uint8>(), color.at(3).get<uint8>());
    group_definition.strength =
        float_from_bits(definition.at("strength_bits").get<std::uint32_t>());
    group_definition.lifetime =
        float_from_bits(definition.at("lifetime_bits").get<std::uint32_t>());
    if (!definition.at("maybe_stride_bits").is_null()) {
      group_definition.stride = float_from_bits(
          definition.at("maybe_stride_bits").get<std::uint32_t>());
    }
    const auto& destination = definition.at("destination");
    const auto append = destination.at("kind") == "append_to";
    if (append) group_definition.group = group(destination.at("target_group_id")).group;

    std::vector<std::unique_ptr<b2Shape>> owned_shapes;
    std::vector<const b2Shape*> shape_pointers;
    std::vector<b2Vec2> positions;
    const auto& source = definition.at("source");
    const auto source_kind = source.at("kind").get<std::string>();
    if (source_kind == "explicit") {
      for (const auto& position : source.at("positions")) {
        positions.push_back(phase10_vector(position));
      }
      group_definition.particleCount = static_cast<int32>(positions.size());
      group_definition.positionData = positions.data();
    } else if (source_kind == "filled") {
      for (const auto& raw_shape : source.at("shapes")) {
        owned_shapes.push_back(shape(raw_shape));
        shape_pointers.push_back(owned_shapes.back().get());
      }
      group_definition.shapes = shape_pointers.data();
      group_definition.shapeCount = static_cast<int32>(shape_pointers.size());
    } else {
      owned_shapes.push_back(shape(source.at("shape")));
      group_definition.shape = owned_shapes.back().get();
    }

    auto* created = owner.system->CreateParticleGroup(group_definition);
    if (created == nullptr) {
      throw std::runtime_error("pinned system rejected Phase 10 group creation");
    }
    const auto created_count = owner.system->GetParticleCount() - before_count;
    if (created_count != static_cast<int32>(definition.at("member_ids").size())) {
      throw std::runtime_error("Phase 10 source produced an unexpected member count");
    }
    std::vector<const b2ParticleHandle*> created_handles;
    for (int32 index = 0; index < owner.system->GetParticleCount(); ++index) {
      const auto* handle = owner.system->GetParticleHandleFromIndex(index);
      if (handle == nullptr) {
        throw std::runtime_error("failed to bind Phase 10 particle identity");
      }
      if (std::find(prior_handles.begin(), prior_handles.end(), handle) ==
          prior_handles.end()) {
        created_handles.push_back(handle);
      }
    }
    if (created_handles.size() != definition.at("member_ids").size()) {
      throw std::runtime_error("Phase 10 created-handle count is misaligned");
    }
    for (std::size_t offset = 0; offset < definition.at("member_ids").size(); ++offset) {
      const auto particle_id =
          definition.at("member_ids").at(offset).get<std::string>();
      auto token = std::make_unique<std::string>(particle_id);
      const auto dense_index = created_handles.at(offset)->GetIndex();
      owner.system->GetUserDataBuffer()[dense_index] = token.get();
      particles_.push_back(
          {particle_id,
           definition.at("system_id").get<std::string>(), owner.system,
           created_handles.at(offset)});
      particle_tokens_.push_back(std::move(token));
    }
    provenance_ = definition.at("provenance");
    if (!append) {
      groups_.push_back(
          {definition.at("group_id").get<std::string>(),
           definition.at("system_id").get<std::string>(), created});
      const auto event = add_event(
          "group_created", definition.at("system_id"), definition.at("group_id"),
          nullptr, nullptr, nullptr);
      upsert_witness(
          "group_create", "activation",
          {{"kind", "occurrence"}, {"event_ordinal", event}});
    } else {
      upsert_witness(
          "group_append", "activation",
          {{"kind", "count"}, {"value", created_count}});
    }
  }

  void execute_group(const Json& operation) {
    const auto kind = operation.at("kind").get<std::string>();
    if (kind == "create_group") return create_group(operation.at("definition"));
    if (kind == "join_groups") {
      auto& target = group(operation.at("target_group_id"));
      const auto target_id = target.id;
      const auto target_system_id = target.system_id;
      auto* target_group = target.group;
      const auto source_id = operation.at("source_group_id").get<std::string>();
      auto& source = group(operation.at("source_group_id"));
      suppress_group_destroy_event_ = true;
      target_group->GetParticleSystem()->JoinParticleGroups(target_group, source.group);
      suppress_group_destroy_event_ = false;
      groups_.erase(
          std::remove_if(
              groups_.begin(), groups_.end(),
              [&](const auto& binding) { return binding.id == source_id; }),
          groups_.end());
      const auto event = add_event(
          "groups_joined", target_system_id, target_id, nullptr, nullptr, nullptr);
      upsert_witness(
          "group_join", "activation",
          {{"kind", "occurrence"}, {"event_ordinal", event}});
      return;
    }
    if (kind == "split_group") {
      auto& source = group(operation.at("group_id"));
      const auto source_id = source.id;
      const auto source_system_id = source.system_id;
      auto* source_group = source.group;
      std::vector<b2ParticleGroup*> prior;
      for (auto* candidate = source_group->GetParticleSystem()->GetParticleGroupList();
           candidate != nullptr; candidate = candidate->GetNext()) {
        prior.push_back(candidate);
      }
      source_group->GetParticleSystem()->SplitParticleGroup(source_group);
      std::vector<b2ParticleGroup*> created;
      for (auto* candidate = source_group->GetParticleSystem()->GetParticleGroupList();
           candidate != nullptr; candidate = candidate->GetNext()) {
        if (std::find(prior.begin(), prior.end(), candidate) == prior.end()) {
          created.push_back(candidate);
        }
      }
      std::sort(
          created.begin(), created.end(),
          [](const auto* a, const auto* b) {
            return a->GetBufferIndex() < b->GetBufferIndex();
          });
      if (created.size() != operation.at("created_group_ids").size()) {
        throw std::runtime_error("Phase 10 split produced an unexpected group count");
      }
      for (std::size_t index = 0; index < created.size(); ++index) {
        groups_.push_back(
            {operation.at("created_group_ids").at(index).get<std::string>(),
             source_system_id, created.at(index)});
      }
      const auto event = add_event(
          "group_split", source_system_id, source_id, nullptr, nullptr, nullptr);
      upsert_witness(
          "group_split", "activation",
          {{"kind", "occurrence"}, {"event_ordinal", event}});
      return;
    }
    if (kind == "set_group_flags") {
      auto& target = group(operation.at("group_id"));
      target.group->SetGroupFlags(operation.at("group_flags_bits").get<uint32>());
      upsert_witness(
          "group_flags", "activation",
          {{"kind", "count"}, {"value", operation.at("group_flags_bits")}});
      return;
    }
    if (kind == "destroy_group") {
      auto& target = group(operation.at("group_id"));
      const auto count = target.group->GetParticleCount();
      if (count == 0) {
        const auto target_id = target.id;
        const auto target_system_id = target.system_id;
        target.group = nullptr;
        const auto event = add_event(
            "group_destroyed", target_system_id, target_id, nullptr, nullptr, nullptr);
        upsert_witness(
            "group_destroy", "activation",
            {{"kind", "occurrence"}, {"event_ordinal", event}});
        return;
      }
      target.group->DestroyParticles(true);
      upsert_witness(
          "group_destroy", "activation",
          {{"kind", "count"}, {"value", count}});
      return;
    }
    if (kind == "step") {
      velocity_before_.clear();
      for (const auto& binding : particles_) {
        if (binding.handle != nullptr &&
            binding.handle->GetIndex() != b2_invalidParticleIndex) {
          velocity_before_.emplace(
              binding.id,
              binding.system->GetVelocityBuffer()[binding.handle->GetIndex()]);
        }
      }
      world_.Step(
          float_from_bits(operation.at("timestep_bits").get<std::uint32_t>()),
          static_cast<int32>(operation.at("velocity_iterations").get<std::uint32_t>()),
          static_cast<int32>(operation.at("position_iterations").get<std::uint32_t>()),
          static_cast<int32>(operation.at("particle_iterations").get<std::uint32_t>()));
      refresh_particle_handles();
      discard_dead();
      return;
    }
    if (kind == "inspect_state") return inspect();
    throw std::runtime_error("unsupported Phase 10 execution operation");
  }

  void create_body(const Json& action) {
    const auto id = action.at("body_id").get<std::string>();
    const auto& raw = body_declarations_.at(id);
    b2BodyDef definition;
    const auto kind = raw.at("body_kind").get<std::string>();
    definition.type = kind == "static" ? b2_staticBody
                      : kind == "kinematic" ? b2_kinematicBody
                                             : b2_dynamicBody;
    definition.position = phase10_vector(raw.at("transform").at("position"));
    definition.angle = float_from_bits(
        raw.at("transform").at("angle_bits").get<std::uint32_t>());
    definition.active = raw.at("active").get<bool>();
    auto* body = world_.CreateBody(&definition);
    if (body == nullptr || !bodies_.emplace(id, body).second) {
      throw std::runtime_error("pinned world failed to create Phase 10 body");
    }
  }

  void create_fixture(const Json& action) {
    const auto id = action.at("fixture_id").get<std::string>();
    const auto& raw = fixture_declarations_.at(id);
    auto* body = bodies_.at(raw.at("owner_body_id").get<std::string>());
    b2FixtureDef definition;
    auto owned_shape = shape(raw.at("shape"));
    definition.shape = owned_shape.get();
    definition.density = float_from_bits(raw.at("density_bits").get<std::uint32_t>());
    definition.friction = float_from_bits(raw.at("friction_bits").get<std::uint32_t>());
    definition.restitution = float_from_bits(raw.at("restitution_bits").get<std::uint32_t>());
    definition.isSensor = raw.at("sensor").get<bool>();
    const auto& filter = raw.at("filter");
    definition.filter.categoryBits = filter.at("category_bits").get<std::uint16_t>();
    definition.filter.maskBits = filter.at("mask_bits").get<std::uint16_t>();
    definition.filter.groupIndex = filter.at("group_index").get<std::int16_t>();
    auto* fixture = body->CreateFixture(&definition);
    if (fixture == nullptr || !fixtures_.emplace(id, fixture).second) {
      throw std::runtime_error("pinned body failed to create Phase 10 fixture");
    }
  }

  void destroy_fixture(const Json& action) {
    const auto id = action.at("fixture_id").get<std::string>();
    const auto found = fixtures_.find(id);
    if (found == fixtures_.end()) return;
    found->second->GetBody()->DestroyFixture(found->second);
    fixtures_.erase(found);
  }

  void destroy_body(const Json& action) {
    const auto id = action.at("body_id").get<std::string>();
    const auto found = bodies_.find(id);
    if (found == bodies_.end()) return;
    auto* doomed = found->second;
    for (auto it = fixtures_.begin(); it != fixtures_.end();) {
      it = it->second->GetBody() == doomed ? fixtures_.erase(it) : std::next(it);
    }
    world_.DestroyBody(doomed);
    bodies_.erase(found);
  }
