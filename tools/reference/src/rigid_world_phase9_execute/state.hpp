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

  Json semantic_body_ids() const {
    Json ids = Json::array();
    for (const auto& declaration : timeline_.at("bodies")) {
      const auto id = declaration.at("body_id").get<std::string>();
      if (bodies_.count(id)) ids.push_back(id);
    }
    return ids;
  }

  std::string semantic_particle_id(
      const b2ParticleSystem* system,
      int32 index) const {
    const auto found = std::find_if(
        particles_.begin(), particles_.end(), [&](const auto& item) {
          return item.second.system == system && item.second.handle != nullptr &&
                 item.second.handle->GetIndex() == index;
        });
    if (found == particles_.end()) {
      throw std::runtime_error("Phase 9 contact particle has no semantic identity");
    }
    return found->first;
  }

  std::string semantic_system_id(const b2ParticleSystem* system) const {
    const auto found = std::find_if(
        systems_.begin(), systems_.end(),
        [&](const auto& item) { return item.second->system == system; });
    if (found == systems_.end()) {
      throw std::runtime_error("Phase 9 occurrence has no semantic system identity");
    }
    return found->first;
  }

  std::string semantic_body_id(const b2Body* body) const {
    const auto found = std::find_if(
        bodies_.begin(), bodies_.end(),
        [&](const auto& item) { return item.second == body; });
    if (found == bodies_.end()) {
      throw std::runtime_error("Phase 9 body contact has no semantic body identity");
    }
    return found->first;
  }

  std::string semantic_fixture_id(const b2Fixture* fixture) const {
    const auto found = std::find_if(
        fixtures_.begin(), fixtures_.end(),
        [&](const auto& item) { return item.second == fixture; });
    if (found == fixtures_.end()) {
      throw std::runtime_error("Phase 9 body contact has no semantic fixture identity");
    }
    return found->first;
  }

  void observe_lifecycle(
      std::string_view kind,
      const std::string& system_id,
      Json maybe_particle_id = nullptr) {
    Json occurrence{
        {"ordinal", next_occurrence_ordinal_++},
        {"kind", std::string(kind)},
        {"system_id", system_id},
        {"maybe_particle_id", std::move(maybe_particle_id)},
        {"maybe_other_particle_id", nullptr},
        {"maybe_fixture_id", nullptr}};
    observations_.push_back(
        {{"kind", "particle"},
         {"observation",
          {{"kind", "lifecycle"},
           {"occurrence", std::move(occurrence)}}}});
  }

  void record_occurrence(
      std::string_view kind,
      const std::string& system_id,
      Json maybe_particle_id = nullptr,
      Json maybe_other_particle_id = nullptr,
      Json maybe_fixture_id = nullptr) {
    occurrences_.push_back(
        {{"ordinal", occurrences_.size()},
         {"kind", std::string(kind)},
         {"system_id", system_id},
         {"maybe_particle_id", std::move(maybe_particle_id)},
         {"maybe_other_particle_id", std::move(maybe_other_particle_id)},
         {"maybe_fixture_id", std::move(maybe_fixture_id)}});
  }

  void inspect_occurrence(const Json& raw_index) {
    const auto index = raw_index.get<std::size_t>();
    if (index >= occurrences_.size()) {
      throw std::runtime_error("unknown Phase 9 occurrence index");
    }
    observations_.push_back(
        {{"kind", "particle"},
         {"observation",
          {{"kind", "lifecycle"}, {"occurrence", occurrences_.at(index)}}}});
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

  b2Fixture& fixture(const Json& raw_id) {
    const auto found = fixtures_.find(raw_id.get<std::string>());
    if (found == fixtures_.end()) {
      throw std::runtime_error("Phase 9 coupling fixture is not live");
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
