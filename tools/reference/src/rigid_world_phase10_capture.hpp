// TimelineExecution semantic capture methods. Included inside the private class.

  std::uint32_t add_event(
      std::string_view kind,
      const Json& system_id,
      Json maybe_group_id,
      Json maybe_particle_id,
      Json maybe_other_particle_id,
      Json maybe_body_id) {
    const auto ordinal = static_cast<std::uint32_t>(events_.size());
    events_.push_back(
        {{"ordinal", ordinal},
         {"kind", std::string(kind)},
         {"system_id", system_id},
         {"maybe_group_id", std::move(maybe_group_id)},
         {"maybe_particle_id", std::move(maybe_particle_id)},
         {"maybe_other_particle_id", std::move(maybe_other_particle_id)},
         {"maybe_body_id", std::move(maybe_body_id)}});
    return ordinal;
  }

  void upsert_witness(
      std::string_view leaf,
      std::string_view role,
      Json observation) {
    const auto found = std::find_if(
        witnesses_.begin(), witnesses_.end(), [&](const auto& witness) {
          return witness.at("behavior_leaf") == leaf && witness.at("role") == role;
        });
    if (found != witnesses_.end()) {
      (*found)["observation"] = std::move(observation);
      return;
    }
    witnesses_.push_back(
        {{"ordinal", witnesses_.size()},
         {"behavior_leaf", std::string(leaf)},
         {"role", std::string(role)},
         {"observation", std::move(observation)}});
  }

  Json capture_groups() const {
    Json result = Json::array();
    for (const auto& binding : groups_) {
      if (binding.group == nullptr) continue;
      Json members = Json::array();
      const auto first = binding.group->GetBufferIndex();
      const auto end = first + binding.group->GetParticleCount();
      for (int32 index = first; index < end; ++index) {
        if ((binding.group->GetParticleSystem()->GetFlagsBuffer()[index] &
             b2_zombieParticle) != 0U) {
          continue;
        }
        members.push_back(semantic_particle_id(binding.group->GetParticleSystem(), index));
      }
      const auto transform = binding.group->GetTransform();
      result.push_back(
          {{"ordinal", result.size()},
           {"group_id", binding.id},
           {"system_id", binding.system_id},
           {"member_ids", std::move(members)},
           {"group_flags_bits", binding.group->GetGroupFlags()},
           {"transform",
            {{"position", encode_rigid_vector(transform.p)},
             {"angle_bits", bits_from_float(transform.q.GetAngle())}}},
           {"center", encode_rigid_vector(binding.group->GetCenter())},
           {"linear_velocity", encode_rigid_vector(binding.group->GetLinearVelocity())},
           {"angular_velocity_bits", bits_from_float(binding.group->GetAngularVelocity())},
           {"mass_bits", bits_from_float(binding.group->GetMass())},
           {"inertia_bits", bits_from_float(binding.group->GetInertia())},
           {"maybe_depths_bits", nullptr}});
    }
    return result;
  }

  Json capture_particles(const Json& groups) const {
    Json result = Json::array();
    for (const auto& group_snapshot : groups) {
      const auto& binding = group(group_snapshot.at("group_id"));
      for (const auto& raw_id : group_snapshot.at("member_ids")) {
        const auto id = raw_id.get<std::string>();
        const auto found = std::find_if(
            particles_.begin(), particles_.end(),
            [&](const auto& candidate) { return candidate.id == id; });
        if (found == particles_.end() || found->handle == nullptr) {
          throw std::runtime_error("Phase 10 group member has no semantic binding");
        }
        const auto index = found->handle->GetIndex();
        const auto color = found->system->GetColorBuffer()[index];
        result.push_back(
            {{"particle_id", found->id},
             {"system_id", found->system_id},
             {"group_id", binding.id},
             {"position", encode_rigid_vector(found->system->GetPositionBuffer()[index])},
             {"velocity", encode_rigid_vector(found->system->GetVelocityBuffer()[index])},
             {"flags_bits", found->system->GetFlagsBuffer()[index]},
             {"color", Json::array({color.r, color.g, color.b, color.a})},
             {"weight_bits",
              bits_from_float(
                  found->system->GetWeightBuffer() == nullptr
                      ? 0.0F
                      : found->system->GetWeightBuffer()[index])}});
      }
    }
    return result;
  }

  Json capture_pairs() const {
    Json result = Json::array();
    for (const auto& [system_id, state] : systems_) {
      static_cast<void>(system_id);
      for (int32 index = 0; index < state->system->GetPairCount(); ++index) {
        const auto& pair = state->system->GetPairs()[index];
        const auto* particle_a = particle_binding(state->system, pair.indexA);
        const auto* particle_b = particle_binding(state->system, pair.indexB);
        if (particle_a == nullptr || particle_b == nullptr) continue;
        result.push_back(
            {{"ordinal", result.size()},
             {"particle_a_id", particle_a->id},
             {"particle_b_id", particle_b->id},
             {"flags_bits", pair.flags},
             {"strength_bits", bits_from_float(pair.strength)},
             {"distance_bits", bits_from_float(pair.distance)}});
      }
    }
    return result;
  }

  Json capture_triads() const {
    Json result = Json::array();
    for (const auto& [system_id, state] : systems_) {
      static_cast<void>(system_id);
      for (int32 index = 0; index < state->system->GetTriadCount(); ++index) {
        const auto& triad = state->system->GetTriads()[index];
        const auto* particle_a = particle_binding(state->system, triad.indexA);
        const auto* particle_b = particle_binding(state->system, triad.indexB);
        const auto* particle_c = particle_binding(state->system, triad.indexC);
        if (particle_a == nullptr || particle_b == nullptr || particle_c == nullptr) {
          continue;
        }
        result.push_back(
            {{"ordinal", result.size()},
             {"particle_a_id", particle_a->id},
             {"particle_b_id", particle_b->id},
             {"particle_c_id", particle_c->id},
             {"flags_bits", triad.flags},
             {"strength_bits", bits_from_float(triad.strength)},
             {"pa", encode_rigid_vector(triad.pa)},
             {"pb", encode_rigid_vector(triad.pb)},
             {"pc", encode_rigid_vector(triad.pc)},
             {"ka_bits", bits_from_float(triad.ka)},
             {"kb_bits", bits_from_float(triad.kb)},
             {"kc_bits", bits_from_float(triad.kc)},
             {"s_bits", bits_from_float(triad.s)}});
      }
    }
    return result;
  }

  Json capture_particle_contacts() const {
    Json result = Json::array();
    for (const auto& [system_id, state] : systems_) {
      for (int32 index = 0; index < state->system->GetContactCount(); ++index) {
        const auto& contact = state->system->GetContacts()[index];
        const auto* particle_a = particle_binding(state->system, contact.GetIndexA());
        const auto* particle_b = particle_binding(state->system, contact.GetIndexB());
        if (particle_a == nullptr || particle_b == nullptr) continue;
        result.push_back(
            {{"ordinal", result.size()},
             {"system_id", system_id},
             {"particle_a_id", particle_a->id},
             {"particle_b_id", particle_b->id},
             {"flags_bits", contact.GetFlags()},
             {"weight_bits", bits_from_float(contact.GetWeight())},
             {"normal", encode_rigid_vector(contact.GetNormal())}});
      }
    }
    return result;
  }

  Json capture_body_contacts() const {
    Json result = Json::array();
    for (const auto& [system_id, state] : systems_) {
      for (int32 index = 0; index < state->system->GetBodyContactCount(); ++index) {
        const auto& contact = state->system->GetBodyContacts()[index];
        const auto* particle = particle_binding(state->system, contact.index);
        if (particle == nullptr) continue;
        result.push_back(
            {{"ordinal", result.size()},
             {"system_id", system_id},
             {"particle_id", particle->id},
             {"body_id", semantic_body_id(contact.body)},
             {"fixture_id", semantic_fixture_id(contact.fixture)},
             {"weight_bits", bits_from_float(contact.weight)},
             {"normal", encode_rigid_vector(contact.normal)},
             {"mass_bits", bits_from_float(contact.mass)}});
      }
    }
    return result;
  }

  void capture_behavior_witnesses(
      const Json& particles,
      const Json& pairs,
      const Json& triads,
      const Json& body_contacts) {
    std::uint32_t flags = 0;
    bool water = false;
    for (const auto& particle : particles) {
      const auto bits = particle.at("flags_bits").get<std::uint32_t>();
      flags |= bits;
      water = water || bits == 0U;
    }
    const std::vector<std::pair<std::string, std::uint32_t>> leaves{
        {"zombie", b2_zombieParticle}, {"wall", b2_wallParticle},
        {"spring", b2_springParticle}, {"elastic", b2_elasticParticle},
        {"viscous", b2_viscousParticle}, {"powder", b2_powderParticle},
        {"tensile", b2_tensileParticle}, {"color_mixing", b2_colorMixingParticle},
        {"barrier", b2_barrierParticle}, {"static_pressure", b2_staticPressureParticle},
        {"reactive", b2_reactiveParticle}, {"repulsive", b2_repulsiveParticle}};
    upsert_witness(
        "water", water ? "activation" : "control",
        water ? Json{{"kind", "flag_activated"}, {"flags_bits", 0U}}
              : Json{{"kind", "control_unchanged"}});
    for (const auto& [leaf, bits] : leaves) {
      const auto active = (flags & bits) != 0U;
      upsert_witness(
          leaf, active ? "activation" : "control",
          active ? Json{{"kind", "flag_activated"}, {"flags_bits", bits}}
                 : Json{{"kind", "control_unchanged"}});
    }
    std::uint32_t group_flags = 0;
    for (const auto& binding : groups_) group_flags |= binding.group->GetGroupFlags();
    for (const auto& [leaf, bits] :
         std::vector<std::pair<std::string, std::uint32_t>>{
             {"solid_group", b2_solidParticleGroup},
             {"rigid_group", b2_rigidParticleGroup}}) {
      const auto active = (group_flags & bits) != 0U;
      upsert_witness(
          leaf, active ? "activation" : "control",
          active ? Json{{"kind", "count"}, {"value", 1U}}
                 : Json{{"kind", "control_unchanged"}});
    }
    const auto body_active = !body_contacts.empty();
    upsert_witness(
        "body_interaction", body_active ? "activation" : "control",
        body_active ? Json{{"kind", "count"}, {"value", body_contacts.size()}}
                    : Json{{"kind", "control_unchanged"}});
    for (const auto* leaf : {"spring", "elastic", "reactive"}) {
      upsert_witness(
          leaf, "interaction",
          {{"kind", "topology"},
           {"pair_count", pairs.size()},
           {"triad_count", triads.size()}});
    }
    const auto velocity_particle = std::find_if(
        particles_.begin(), particles_.end(), [&](const auto& binding) {
          return binding.handle != nullptr &&
                 velocity_before_.count(binding.id) != 0U;
        });
    if (velocity_particle == particles_.end()) return;
    const auto dense_index = velocity_particle->handle->GetIndex();
    for (const auto& [leaf, bits] : leaves) {
      if ((flags & bits) == 0U) continue;
      upsert_witness(
          leaf, "interaction",
          {{"kind", "particle_velocity"},
           {"particle_id", velocity_particle->id},
           {"before", encode_rigid_vector(velocity_before_.at(velocity_particle->id))},
           {"after",
            encode_rigid_vector(
                velocity_particle->system->GetVelocityBuffer()[dense_index])}});
    }
  }

  void inspect() {
    auto groups = capture_groups();
    auto particles = capture_particles(groups);
    auto pairs = capture_pairs();
    auto triads = capture_triads();
    auto particle_contacts = capture_particle_contacts();
    auto body_contacts = capture_body_contacts();
    capture_behavior_witnesses(particles, pairs, triads, body_contacts);
    observations_.push_back(
        {{"kind", "particle_group"},
         {"observation",
          {{"kind", "state"},
           {"state",
            {{"provenance", provenance_},
             {"outcome", {{"kind", "completed"}}},
             {"groups", std::move(groups)},
             {"particles", std::move(particles)},
             {"pairs", std::move(pairs)},
             {"triads", std::move(triads)},
             {"particle_contacts", std::move(particle_contacts)},
             {"body_contacts", std::move(body_contacts)},
             {"events", events_},
             {"witnesses", witnesses_}}}}}});
  }
