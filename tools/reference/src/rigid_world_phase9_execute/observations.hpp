  void observe_system(const Json& action) {
    auto& state = system(action.at("system_id"));
    Json particle_ids = Json::array();
    for (const auto& declaration : timeline_.at("particles")) {
      const auto particle_id = declaration.at("particle_id").get<std::string>();
      const auto found = particles_.find(particle_id);
      if (found != particles_.end() && found->second.system == state.system &&
          found->second.handle != nullptr &&
          found->second.handle->GetIndex() != b2_invalidParticleIndex) {
        particle_ids.push_back(particle_id);
      }
    }
    observations_.push_back(
        {{"kind", "particle"},
         {"observation",
          {{"kind", "system"},
           {"system_id", action.at("system_id")},
           {"paused", state.system->GetPaused()},
           {"particle_ids", std::move(particle_ids)}}}});
  }

  void observe_particle(const Json& action) {
    const auto particle_id = action.at("particle_id").get<std::string>();
    auto& state = particle(action.at("particle_id"));
    const auto index = state.handle->GetIndex();
    const auto& declaration = particle_declarations_.at(particle_id);
    const auto color = state.system->GetColorBuffer()[index];
    observations_.push_back(
        {{"kind", "particle"},
         {"observation",
          {{"kind", "particle"},
          {"snapshot",
            {{"particle_id", particle_id},
             {"system_id", declaration.at("system_id")},
             {"position", encode_rigid_vector(state.system->GetPositionBuffer()[index])},
             {"velocity", encode_rigid_vector(state.system->GetVelocityBuffer()[index])},
             {"flags_bits", state.system->GetFlagsBuffer()[index]},
             {"color", Json::array({color.r, color.g, color.b, color.a})},
             {"weight_bits", bits_from_float(state.system->GetWeightBuffer()[index])},
             {"force", encode_rigid_vector(particle_forces_.at(particle_id))},
             {"pending_destruction", false}}}}}});
  }

  void observe_particle_contact(const Json& action) {
    auto& state = system(action.at("system_id"));
    const auto index = action.at("contact_index").get<std::size_t>();
    if (index >= static_cast<std::size_t>(state.system->GetContactCount())) {
      throw std::runtime_error("Phase 9 particle contact index is not live");
    }
    const auto& contact = state.system->GetContacts()[index];
    observations_.push_back(
        {{"kind", "particle"},
         {"observation",
          {{"kind", "particle_contact"},
           {"contact",
            {{"system_id", action.at("system_id")},
             {"particle_a_id",
              semantic_particle_id(state.system, contact.GetIndexA())},
             {"particle_b_id",
              semantic_particle_id(state.system, contact.GetIndexB())},
             {"flags_bits", contact.GetFlags()},
             {"weight_bits", bits_from_float(contact.GetWeight())},
             {"normal", encode_rigid_vector(contact.GetNormal())}}}}}});
  }

  void observe_body_contact(const Json& action) {
    auto& state = system(action.at("system_id"));
    const auto index = action.at("contact_index").get<std::size_t>();
    if (index >= static_cast<std::size_t>(state.system->GetBodyContactCount())) {
      throw std::runtime_error("Phase 9 body contact index is not live");
    }
    const auto& contact = state.system->GetBodyContacts()[index];
    observations_.push_back(
        {{"kind", "particle"},
         {"observation",
          {{"kind", "body_contact"},
           {"contact",
            {{"system_id", action.at("system_id")},
             {"particle_id", semantic_particle_id(state.system, contact.index)},
             {"body_id", semantic_body_id(contact.body)},
             {"fixture_id", semantic_fixture_id(contact.fixture)},
             {"weight_bits", bits_from_float(contact.weight)},
             {"normal", encode_rigid_vector(contact.normal)},
             {"mass_bits", bits_from_float(contact.mass)}}}}}});
  }

  void observe_statistics(const Json& action) {
    auto& state = system(action.at("system_id"));
    const auto count = state.system->GetParticleCount();
    Json stuck = Json::array();
    for (int32 index = 0; index < state.system->GetStuckCandidateCount(); ++index) {
      const auto dense = state.system->GetStuckCandidates()[index];
      const auto found = std::find_if(particles_.begin(), particles_.end(), [&](const auto& item) {
        return item.second.system == state.system && item.second.handle->GetIndex() == dense;
      });
      if (found != particles_.end()) stuck.push_back(found->first);
    }
    observations_.push_back(
        {{"kind", "particle"},
         {"observation",
          {{"kind", "statistics"},
           {"statistics",
            {{"maybe_system_id", action.at("system_id")},
             {"system_count", static_cast<std::uint32_t>(systems_.size())},
             {"particle_count", static_cast<std::uint32_t>(count)},
             {"pending_particle_count", 0U},
             {"particle_contact_count", static_cast<std::uint32_t>(state.system->GetContactCount())},
             {"body_contact_count", static_cast<std::uint32_t>(state.system->GetBodyContactCount())},
             {"stuck_particle_ids", std::move(stuck)},
             {"collision_energy_bits", bits_from_float(state.system->ComputeCollisionEnergy())},
             {"declared_capacity", static_cast<std::uint32_t>(state.declared_capacity)},
             {"effective_capacity", static_cast<std::uint32_t>(state.declared_capacity)}}}}}});
  }

  void observe_query(const Json& action) {
    QueryCollector collector(
        particles_, action.value("control", std::string{"continue"}));
    b2AABB aabb;
    aabb.lowerBound = phase9_vector(action.at("lower"));
    aabb.upperBound = phase9_vector(action.at("upper"));
    if (action.at("system_id").is_null()) {
      world_.QueryAABB(&collector, aabb);
    } else {
      system(action.at("system_id")).system->QueryAABB(&collector, aabb);
    }
    observations_.push_back(
        {{"kind", "particle"},
         {"observation",
          {{"kind", "query"},
           {"terminated", collector.terminated},
           {"particle_ids", collector.ids}}}});
  }

  void observe_ray(const Json& action) {
    RayCollector collector(
        particles_, action.value("control", std::string{"continue"}));
    const auto start = phase9_vector(action.at("start"));
    const auto end = phase9_vector(action.at("end"));
    if (action.at("system_id").is_null()) {
      world_.RayCast(&collector, start, end);
    } else {
      system(action.at("system_id")).system->RayCast(&collector, start, end);
    }
    observations_.push_back(
        {{"kind", "particle"},
         {"observation",
          {{"kind", "ray_cast"},
           {"terminated", collector.terminated},
           {"particle_ids", collector.ids},
           {"fractions_bits", collector.fractions}}}});
  }

  void discard_dead_particles() {
    for (auto it = particles_.begin(); it != particles_.end();) {
      it = it->second.handle == nullptr ||
                   it->second.handle->GetIndex() == b2_invalidParticleIndex
               ? particles_.erase(it)
               : std::next(it);
    }
  }
