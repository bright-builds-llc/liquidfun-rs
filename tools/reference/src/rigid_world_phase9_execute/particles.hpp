  void create_system(const Json& action) {
    const auto id = action.at("system_id").get<std::string>();
    const auto& raw = system_declarations_.at(id);
    b2ParticleSystemDef definition;
    definition.strictContactCheck = raw.at("strict_contact_check").get<bool>();
    definition.density =
        float_from_bits(raw.at("density_bits").get<std::uint32_t>());
    definition.gravityScale =
        float_from_bits(raw.at("gravity_scale_bits").get<std::uint32_t>());
    definition.radius = float_from_bits(raw.at("radius_bits").get<std::uint32_t>());
    definition.dampingStrength =
        float_from_bits(raw.at("damping_bits").get<std::uint32_t>());
    definition.destroyByAge = raw.at("destruction_by_age").get<bool>();
    definition.lifetimeGranularity =
        float_from_bits(raw.at("lifetime_granularity_bits").get<std::uint32_t>());
    if (!raw.at("maximum_count").is_null()) {
      definition.maxCount = raw.at("maximum_count").get<int32>();
    }
    auto state = std::make_unique<SystemState>();
    state->system = world_.CreateParticleSystem(&definition);
    if (state->system == nullptr) {
      throw std::runtime_error("pinned world failed to create Phase 9 system");
    }
    const auto& buffer = raw.at("buffer_mode");
    state->fixed = buffer.at("kind") == "fixed";
    state->declared_capacity = state->fixed
                                   ? buffer.at("capacity").get<std::size_t>()
                                   : raw.at("maximum_count").is_null()
                                         ? static_cast<std::size_t>(
                                               std::numeric_limits<int32>::max())
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
      throw std::runtime_error("duplicate live Phase 9 system");
    }
  }

  bool create_particle(const Json& action) {
    const auto id = action.at("particle_id").get<std::string>();
    const auto& raw = particle_declarations_.at(id);
    auto& owner = system(raw.at("system_id"));
    b2ParticleDef definition;
    definition.position = phase9_vector(raw.at("position"));
    definition.velocity = phase9_vector(raw.at("velocity"));
    definition.flags = raw.at("flags_bits").get<uint32>();
    const auto& color = raw.at("color");
    definition.color = b2ParticleColor(
        color.at(0).get<uint8>(), color.at(1).get<uint8>(),
        color.at(2).get<uint8>(), color.at(3).get<uint8>());
    definition.lifetime =
        float_from_bits(raw.at("lifetime_bits").get<std::uint32_t>());
    std::vector<std::pair<std::string, bool>> prior_particles;
    for (const auto& [particle_id, particle_state] : particles_) {
      if (particle_state.system != owner.system || particle_state.handle == nullptr ||
          particle_state.handle->GetIndex() == b2_invalidParticleIndex) {
        continue;
      }
      const auto prior_index = particle_state.handle->GetIndex();
      const auto requested =
          (owner.system->GetFlagsBuffer()[prior_index] &
           b2_destructionListenerParticle) != 0U;
      prior_particles.emplace_back(particle_id, requested);
    }
    const auto index = owner.system->CreateParticle(definition);
    if (index == b2_invalidParticleIndex) {
      throw std::runtime_error("pinned system rejected Phase 9 particle creation");
    }
    const auto* handle = owner.system->GetParticleHandleFromIndex(index);
    if (handle == nullptr) {
      throw std::runtime_error("failed to assign stable Phase 9 particle identity");
    }
    for (auto it = particles_.begin(); it != particles_.end();) {
      it = it->second.system == owner.system && it->second.handle != nullptr &&
                   it->second.handle->GetIndex() == index
               ? particles_.erase(it)
               : std::next(it);
    }
    if (!particles_.emplace(id, ParticleState{owner.system, handle}).second) {
      throw std::runtime_error("failed to assign stable Phase 9 particle identity");
    }
    particle_forces_.emplace(id, b2Vec2_zero);
    std::vector<std::string> requested_evictions;
    for (const auto& [particle_id, requested] : prior_particles) {
      const auto found = particles_.find(particle_id);
      if (requested && found == particles_.end()) {
        requested_evictions.push_back(particle_id);
      }
    }
    if (requested_evictions.size() > 1) {
      throw std::runtime_error("one Phase 9 creation emitted multiple occurrences");
    }
    if (requested_evictions.empty()) return false;
    observe_lifecycle(
        "particle_destroyed", raw.at("system_id").get<std::string>(),
        requested_evictions.front());
    return true;
  }

  void apply_range(const Json& action, bool impulse) {
    const auto count = action.at("particle_ids").size();
    const auto vector = phase9_vector(action.at(impulse ? "impulse" : "force"));
    const auto distributed = (1.0F / static_cast<float32>(count)) * vector;
    for (const auto& raw_id : action.at("particle_ids")) {
      auto& value = particle(raw_id);
      if (impulse) {
        value.system->ParticleApplyLinearImpulse(value.handle->GetIndex(), distributed);
      } else {
        value.system->ParticleApplyForce(value.handle->GetIndex(), distributed);
        particle_forces_.at(raw_id.get<std::string>()) += distributed;
      }
    }
  }

  void execute(const Json& action) {
    const auto kind = action.at("kind").get<std::string>();
    bool observed = false;
    if (kind == "create_system") create_system(action);
    else if (kind == "destroy_system") {
      const auto id = action.at("system_id").get<std::string>();
      auto found = systems_.find(id);
      if (found == systems_.end()) throw std::runtime_error("Phase 9 system is not live");
      auto* doomed = found->second->system;
      world_.DestroyParticleSystem(doomed);
      systems_.erase(found);
      for (auto it = particles_.begin(); it != particles_.end();) {
        it = it->second.system == doomed ? particles_.erase(it) : std::next(it);
      }
      observe_lifecycle("system_destroyed", id);
      observed = true;
    } else if (kind == "create_particle") observed = create_particle(action);
    else if (kind == "inspect_system") {
      observe_system(action);
      observed = true;
    } else if (kind == "inspect_particle") {
      observe_particle(action);
      observed = true;
    } else if (kind == "inspect_particle_contact") {
      observe_particle_contact(action);
      observed = true;
    } else if (kind == "inspect_body_contact") {
      observe_body_contact(action);
      observed = true;
    } else if (kind == "inspect_occurrence") {
      inspect_occurrence(action.at("occurrence_index"));
      observed = true;
    } else if (kind == "set_paused") {
      system(action.at("system_id")).system->SetPaused(action.at("paused").get<bool>());
    } else if (kind == "set_position") {
      auto& value = particle(action.at("particle_id"));
      value.system->GetPositionBuffer()[value.handle->GetIndex()] =
          phase9_vector(action.at("position"));
    } else if (kind == "set_velocity") {
      auto& value = particle(action.at("particle_id"));
      value.system->GetVelocityBuffer()[value.handle->GetIndex()] =
          phase9_vector(action.at("velocity"));
    } else if (kind == "mark_for_destruction") {
      auto& value = particle(action.at("particle_id"));
      const auto index = value.handle->GetIndex();
      const auto requested =
          (value.system->GetFlagsBuffer()[index] & b2_destructionListenerParticle) != 0U;
      value.system->DestroyParticle(index, requested);
    } else if (kind == "compact") {
      auto& owner = system(action.at("system_id"));
      std::vector<std::string> requested_destructions;
      for (const auto& [particle_id, particle_state] : particles_) {
        if (particle_state.system != owner.system || particle_state.handle == nullptr ||
            particle_state.handle->GetIndex() == b2_invalidParticleIndex) {
          continue;
        }
        const auto flags = owner.system->GetFlagsBuffer()[particle_state.handle->GetIndex()];
        if ((flags & b2_zombieParticle) != 0U &&
            (flags & b2_destructionListenerParticle) != 0U) {
          requested_destructions.push_back(particle_id);
        }
      }
      world_.Step(std::numeric_limits<float32>::denorm_min(), 0, 0, 1);
      discard_dead_particles();
      if (requested_destructions.size() > 1) {
        throw std::runtime_error("one Phase 9 compaction emitted multiple occurrences");
      }
      if (!requested_destructions.empty()) {
        observe_lifecycle(
            "particle_destroyed", action.at("system_id").get<std::string>(),
            requested_destructions.front());
        observed = true;
      }
    } else if (kind == "apply_force") apply_range(action, false);
    else if (kind == "apply_impulse") apply_range(action, true);
    else if (kind == "request_statistics") {
      observe_statistics(action);
      observed = true;
    } else if (kind == "query_aabb") {
      observe_query(action);
      observed = true;
    } else if (kind == "ray_cast") {
      observe_ray(action);
      observed = true;
    } else {
      throw std::runtime_error("unsupported Phase 9 execution action");
    }
    if (!observed) observe_mixed_state();
  }
