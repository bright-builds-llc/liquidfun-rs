  // Phase 7 semantic observation and query-control helpers.
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
  void configured_step(const ConfiguredStep& step) {
    const auto velocity_iterations = static_cast<int32>(step.velocity_iterations);
    const auto position_iterations = static_cast<int32>(step.position_iterations);
    const auto bounded_budget_pause =
        timeline_.family == RigidWitnessFamily::continuous_budget &&
        step.continuous_work_budget == 1 && !budget_pending_;
    const auto original_sub_stepping = world_.GetSubStepping();
    if (bounded_budget_pause) world_.SetSubStepping(true);
    world_.Step(
        float_from_bits(step.timestep),
        velocity_iterations,
        position_iterations,
        1);
    if (bounded_budget_pause) world_.SetSubStepping(original_sub_stepping);
    const auto budget_paused = bounded_budget_pause && world_.GetContactCount() > 0;
    Json outcome;
    if (budget_paused) {
      budget_pending_ = true;
      outcome = {
          {"kind", "partial"},
          {"classification", "continuous_work_budget_exhausted"}};
    } else {
      const auto pending = world_.GetSubStepping() && world_.GetContactCount() > 0 &&
                           !substep_pending_;
      substep_pending_ = pending;
      if (!pending) budget_pending_ = false;
      outcome = {
          {"kind", "completed"},
          {"completion", pending ? "continuous_pending" : "complete"}};
    }
    observations_.push_back({{"kind", "step"}, {"outcome", std::move(outcome)}});
  }

  void query_aabb(const QueryAabb& query) {
    class Callback final : public b2QueryCallback {
     public:
      Callback(
          const std::unordered_map<const b2Fixture*, std::string>& fixture_ids,
          const std::vector<RigidQueryRule>& rules,
          Json& occurrences,
          bool& terminated)
          : fixture_ids_(fixture_ids),
            rules_(rules),
            occurrences_(occurrences),
            terminated_(terminated) {}

      bool ReportFixture(b2Fixture* fixture) override {
        const auto found = fixture_ids_.find(fixture);
        if (found == fixture_ids_.end()) {
          throw std::runtime_error("query returned an unmapped fixture");
        }
        occurrences_.push_back({{"fixture_id", found->second}, {"child_index", 0}});
        const auto rule = std::find_if(
            rules_.begin(), rules_.end(), [&](const auto& candidate) {
              return candidate.target.fixture_id == found->second &&
                     candidate.target.child_index == 0;
            });
        if (rule == rules_.end() ||
            rule->directive == RigidQueryDirective::continue_query) {
          return true;
        }
        terminated_ = true;
        return false;
      }

     private:
      const std::unordered_map<const b2Fixture*, std::string>& fixture_ids_;
      const std::vector<RigidQueryRule>& rules_;
      Json& occurrences_;
      bool& terminated_;
    };

    b2AABB aabb;
    aabb.lowerBound = vector(query.aabb.lower);
    aabb.upperBound = vector(query.aabb.upper);
    Json occurrences = Json::array();
    bool terminated = false;
    Callback callback(fixture_ids_, query.rules, occurrences, terminated);
    world_.QueryAABB(&callback, aabb);
    observations_.push_back(
        {{"kind", "query"},
         {"observation",
          {{"completion", terminated ? "terminated" : "exhausted"},
           {"occurrences", std::move(occurrences)}}}});
  }

  void ray_cast(const RayCast& ray) {
    class Callback final : public b2RayCastCallback {
     public:
      Callback(
          const std::unordered_map<const b2Fixture*, std::string>& fixture_ids,
          const std::vector<RigidRayRule>& rules,
          Json& hits,
          bool& terminated,
          float32& final_max_fraction)
          : fixture_ids_(fixture_ids),
            rules_(rules),
            hits_(hits),
            terminated_(terminated),
            final_max_fraction_(final_max_fraction) {}

      float32 ReportFixture(
          b2Fixture* fixture,
          const b2Vec2& point,
          const b2Vec2& normal,
          float32 fraction) override {
        const auto found = fixture_ids_.find(fixture);
        if (found == fixture_ids_.end()) {
          throw std::runtime_error("ray cast returned an unmapped fixture");
        }
        hits_.push_back(
            {{"fixture_id", found->second},
             {"child_index", 0},
             {"point", encode_rigid_vector(point)},
             {"normal", encode_rigid_vector(normal)},
             {"fraction_bits", bits_from_float(fraction)}});
        const auto rule = std::find_if(
            rules_.begin(), rules_.end(), [&](const auto& candidate) {
              return candidate.target.fixture_id == found->second &&
                     candidate.target.child_index == 0;
            });
        if (rule == rules_.end() ||
            rule->directive.kind == RigidRayDirectiveKind::continue_ray) {
          return -1.0F;
        }
        if (rule->directive.kind == RigidRayDirectiveKind::ignore) return -1.0F;
        if (rule->directive.kind == RigidRayDirectiveKind::clip) {
          const auto candidate = float_from_bits(rule->directive.fraction);
          if (candidate > final_max_fraction_) {
            throw std::runtime_error("ray clip would expand current interval");
          }
          if (candidate < final_max_fraction_) final_max_fraction_ = candidate;
          return candidate;
        }
        terminated_ = true;
        return 0.0F;
      }

     private:
      const std::unordered_map<const b2Fixture*, std::string>& fixture_ids_;
      const std::vector<RigidRayRule>& rules_;
      Json& hits_;
      bool& terminated_;
      float32& final_max_fraction_;
    };

    Json hits = Json::array();
    bool terminated = false;
    float32 final_max_fraction = 1.0F;
    Callback callback(fixture_ids_, ray.rules, hits, terminated, final_max_fraction);
    world_.RayCast(&callback, vector(ray.start), vector(ray.end));
    observations_.push_back(
        {{"kind", "ray_cast"},
         {"observation",
          {{"completion", terminated ? "terminated" : "exhausted"},
           {"final_max_fraction_bits", bits_from_float(final_max_fraction)},
           {"hits", std::move(hits)}}}});
  }
