  void execute(const RigidAction& action) {
    begin_action();
    bool was_step = false;
    std::visit(
        [&](const auto& current) {
          using T = std::decay_t<decltype(current)>;
          if constexpr (std::is_same_v<T, CreateBody>) {
            create_body(current.body_id);
          } else if constexpr (std::is_same_v<T, CreateFixture>) {
            create_fixture(current.fixture_id);
          } else if constexpr (std::is_same_v<T, InspectBody>) {
            static_cast<void>(body(current.body_id));
          } else if constexpr (std::is_same_v<T, InspectFixture>) {
            static_cast<void>(fixture(current.fixture_id));
          } else if constexpr (std::is_same_v<T, SetBodyTransform>) {
            body(current.body_id).SetTransform(
                vector(current.transform.position),
                float_from_bits(current.transform.angle));
          } else if constexpr (std::is_same_v<T, SetBodyType>) {
            body(current.body_id).SetType(body_type(current.kind));
          } else if constexpr (std::is_same_v<T, SetBodyActive>) {
            body(current.body_id).SetActive(current.active);
          } else if constexpr (std::is_same_v<T, SetLinearVelocity>) {
            body(current.body_id).SetLinearVelocity(vector(current.velocity));
            observe_body(current.body_id);
          } else if constexpr (std::is_same_v<T, SetAngularVelocity>) {
            body(current.body_id).SetAngularVelocity(float_from_bits(current.velocity));
            observe_body(current.body_id);
          } else if constexpr (std::is_same_v<T, ApplyForce>) {
            body(current.body_id).ApplyForce(
                vector(current.force), vector(current.point), should_wake(current.wake_policy));
            observe_body(current.body_id);
          } else if constexpr (std::is_same_v<T, ApplyTorque>) {
            body(current.body_id).ApplyTorque(
                float_from_bits(current.torque), should_wake(current.wake_policy));
            observe_body(current.body_id);
          } else if constexpr (std::is_same_v<T, ApplyLinearImpulse>) {
            body(current.body_id).ApplyLinearImpulse(
                vector(current.impulse), vector(current.point), should_wake(current.wake_policy));
            observe_body(current.body_id);
          } else if constexpr (std::is_same_v<T, ApplyAngularImpulse>) {
            body(current.body_id).ApplyAngularImpulse(
                float_from_bits(current.impulse), should_wake(current.wake_policy));
            observe_body(current.body_id);
          } else if constexpr (std::is_same_v<T, SetBodyDamping>) {
            auto& target = body(current.body_id);
            target.SetLinearDamping(float_from_bits(current.linear));
            target.SetAngularDamping(float_from_bits(current.angular));
            observe_body(current.body_id);
          } else if constexpr (std::is_same_v<T, SetGravityScale>) {
            body(current.body_id).SetGravityScale(float_from_bits(current.scale));
            observe_body(current.body_id);
          } else if constexpr (std::is_same_v<T, SetFixedRotation>) {
            body(current.body_id).SetFixedRotation(current.fixed);
            observe_body(current.body_id);
          } else if constexpr (std::is_same_v<T, SetSleepingAllowed>) {
            body(current.body_id).SetSleepingAllowed(current.allowed);
            observe_body(current.body_id);
          } else if constexpr (std::is_same_v<T, SetAwake>) {
            body(current.body_id).SetAwake(current.awake);
            observe_body(current.body_id);
          } else if constexpr (std::is_same_v<T, SetBullet>) {
            body(current.body_id).SetBullet(current.bullet);
            observe_body(current.body_id);
          } else if constexpr (std::is_same_v<T, SetFixtureSensor>) {
            fixture(current.fixture_id).SetSensor(current.sensor);
          } else if constexpr (std::is_same_v<T, SetFixtureMaterial>) {
            auto& target = fixture(current.fixture_id);
            target.SetFriction(float_from_bits(current.friction));
            target.SetRestitution(float_from_bits(current.restitution));
          } else if constexpr (std::is_same_v<T, SetFixtureFilter>) {
            b2Filter filter;
            filter.categoryBits = current.filter.category;
            filter.maskBits = current.filter.mask;
            filter.groupIndex = current.filter.group;
            fixture(current.fixture_id).SetFilterData(filter);
          } else if constexpr (std::is_same_v<T, SetFixtureDensity>) {
            fixture(current.fixture_id).SetDensity(float_from_bits(current.density));
          } else if constexpr (std::is_same_v<T, ResetMassData>) {
            body(current.body_id).ResetMassData();
          } else if constexpr (std::is_same_v<T, SetCustomMassData>) {
            b2MassData data;
            data.mass = float_from_bits(current.mass);
            data.center = vector(current.center);
            data.I = float_from_bits(current.inertia);
            body(current.body_id).SetMassData(&data);
          } else if constexpr (std::is_same_v<T, RigidStep>) {
            was_step = true;
            step_start_contacts_ = contacts();
            const auto timestep = float_from_bits(current.timestep);
            const auto velocity_iterations =
                static_cast<int32>(current.velocity_iterations);
            const auto position_iterations =
                static_cast<int32>(current.position_iterations);
            // The pinned accessor is const-only even though FindNewContacts is
            // public. Invoke the world-owned manager before Collide so touched
            // proxies are processed without advancing simulation time twice.
            auto& contact_manager =
                const_cast<b2ContactManager&>(world_.GetContactManager());
            contact_manager.FindNewContacts();
            world_.Step(timestep, velocity_iterations, position_iterations, 1);
          } else if constexpr (std::is_same_v<T, SetWorldGravity>) {
            world_.SetGravity(vector(current.gravity));
          } else if constexpr (std::is_same_v<T, SetAutomaticForceClearing>) {
            world_.SetAutoClearForces(current.enabled);
          } else if constexpr (std::is_same_v<T, SetWarmStarting>) {
            world_.SetWarmStarting(current.enabled);
          } else if constexpr (std::is_same_v<T, SetContinuousPhysics>) {
            world_.SetContinuousPhysics(current.enabled);
          } else if constexpr (std::is_same_v<T, SetSubStepping>) {
            world_.SetSubStepping(current.enabled);
          } else if constexpr (std::is_same_v<T, ClearForces>) {
            world_.ClearForces();
          } else if constexpr (std::is_same_v<T, ConfiguredStep>) {
            was_step = true;
            step_start_contacts_ = contacts();
            configured_step(current);
          } else if constexpr (std::is_same_v<T, QueryAabb>) {
            query_aabb(current);
          } else if constexpr (std::is_same_v<T, RayCast>) {
            ray_cast(current);
          } else if constexpr (std::is_same_v<T, ShiftOrigin>) {
            world_.ShiftOrigin(vector(current.shift));
            observations_.push_back(
                {{"kind", "origin_shift"},
                 {"shift", encode_rigid_vector(vector(current.shift))}});
          } else if constexpr (std::is_same_v<T, DestroyFixture>) {
            destroy_fixture(current.fixture_id);
          } else if constexpr (std::is_same_v<T, DestroyBody>) {
            destroy_body(current.body_id);
          }
        },
        action);
    end_action(was_step);
  }
