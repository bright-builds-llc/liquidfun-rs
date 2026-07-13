// Closed rigid-world action decoder.
inline RigidAction action(const Json& value) {
  const auto kind = text(member(value, "kind", "action"), "action kind");
  if (kind == "create_body") {
    require_members(value, {"kind", "body_id"}, "create-body action");
    return CreateBody{id(member(value, "body_id", "action"), "body ID")};
  }
  if (kind == "create_fixture") {
    require_members(value, {"kind", "fixture_id"}, "create-fixture action");
    return CreateFixture{id(member(value, "fixture_id", "action"), "fixture ID")};
  }
  if (kind == "inspect_body") {
    require_members(value, {"kind", "body_id"}, "inspect-body action");
    return InspectBody{id(member(value, "body_id", "action"), "body ID")};
  }
  if (kind == "inspect_fixture") {
    require_members(value, {"kind", "fixture_id"}, "inspect-fixture action");
    return InspectFixture{id(member(value, "fixture_id", "action"), "fixture ID")};
  }
  if (kind == "set_body_transform") {
    require_members(value, {"kind", "body_id", "transform"}, "body-transform action");
    return SetBodyTransform{
        id(member(value, "body_id", "action"), "body ID"),
        transform(member(value, "transform", "action"))};
  }
  if (kind == "set_body_type") {
    require_members(value, {"kind", "body_id", "body_kind"}, "body-type action");
    return SetBodyType{
        id(member(value, "body_id", "action"), "body ID"),
        body_kind(member(value, "body_kind", "action"))};
  }
  if (kind == "set_body_active") {
    require_members(value, {"kind", "body_id", "active"}, "body-active action");
    return SetBodyActive{
        id(member(value, "body_id", "action"), "body ID"),
        boolean(member(value, "active", "action"), "active")};
  }
  if (kind == "set_linear_velocity") {
    require_members(value, {"kind", "body_id", "velocity"}, "linear-velocity action");
    return SetLinearVelocity{
        id(member(value, "body_id", "action"), "body ID"),
        vec2(member(value, "velocity", "action"), "linear velocity")};
  }
  if (kind == "set_angular_velocity") {
    require_members(value, {"kind", "body_id", "angular_velocity_bits"}, "angular-velocity action");
    const auto bits = u32(member(value, "angular_velocity_bits", "action"), "angular velocity");
    require_finite(bits, "angular velocity");
    return SetAngularVelocity{id(member(value, "body_id", "action"), "body ID"), bits};
  }
  if (kind == "apply_force") {
    require_members(value, {"kind", "body_id", "force", "point", "wake_policy"}, "apply-force action");
    return ApplyForce{
        id(member(value, "body_id", "action"), "body ID"),
        vec2(member(value, "force", "action"), "force"),
        vec2(member(value, "point", "action"), "force point"),
        wake_policy(member(value, "wake_policy", "action"))};
  }
  if (kind == "apply_torque") {
    require_members(value, {"kind", "body_id", "torque_bits", "wake_policy"}, "apply-torque action");
    const auto bits = u32(member(value, "torque_bits", "action"), "torque");
    require_finite(bits, "torque");
    return ApplyTorque{
        id(member(value, "body_id", "action"), "body ID"), bits,
        wake_policy(member(value, "wake_policy", "action"))};
  }
  if (kind == "apply_linear_impulse") {
    require_members(value, {"kind", "body_id", "impulse", "point", "wake_policy"}, "linear-impulse action");
    return ApplyLinearImpulse{
        id(member(value, "body_id", "action"), "body ID"),
        vec2(member(value, "impulse", "action"), "linear impulse"),
        vec2(member(value, "point", "action"), "impulse point"),
        wake_policy(member(value, "wake_policy", "action"))};
  }
  if (kind == "apply_angular_impulse") {
    require_members(value, {"kind", "body_id", "impulse_bits", "wake_policy"}, "angular-impulse action");
    const auto bits = u32(member(value, "impulse_bits", "action"), "angular impulse");
    require_finite(bits, "angular impulse");
    return ApplyAngularImpulse{
        id(member(value, "body_id", "action"), "body ID"), bits,
        wake_policy(member(value, "wake_policy", "action"))};
  }
  if (kind == "set_body_damping") {
    require_members(value, {"kind", "body_id", "linear_damping_bits", "angular_damping_bits"}, "body-damping action");
    const auto linear = u32(member(value, "linear_damping_bits", "action"), "linear damping");
    const auto angular = u32(member(value, "angular_damping_bits", "action"), "angular damping");
    require_nonnegative(linear, "linear damping");
    require_nonnegative(angular, "angular damping");
    return SetBodyDamping{id(member(value, "body_id", "action"), "body ID"), linear, angular};
  }
  if (kind == "set_gravity_scale") {
    require_members(value, {"kind", "body_id", "gravity_scale_bits"}, "gravity-scale action");
    const auto bits = u32(member(value, "gravity_scale_bits", "action"), "gravity scale");
    require_finite(bits, "gravity scale");
    return SetGravityScale{id(member(value, "body_id", "action"), "body ID"), bits};
  }
  if (kind == "set_fixed_rotation") {
    require_members(value, {"kind", "body_id", "fixed_rotation"}, "fixed-rotation action");
    return SetFixedRotation{
        id(member(value, "body_id", "action"), "body ID"),
        boolean(member(value, "fixed_rotation", "action"), "fixed rotation")};
  }
  if (kind == "set_sleeping_allowed") {
    require_members(value, {"kind", "body_id", "sleeping_allowed"}, "sleeping-allowed action");
    return SetSleepingAllowed{
        id(member(value, "body_id", "action"), "body ID"),
        boolean(member(value, "sleeping_allowed", "action"), "sleeping allowed")};
  }
  if (kind == "set_awake") {
    require_members(value, {"kind", "body_id", "awake"}, "awake action");
    return SetAwake{
        id(member(value, "body_id", "action"), "body ID"),
        boolean(member(value, "awake", "action"), "awake")};
  }
  if (kind == "set_bullet") {
    require_members(value, {"kind", "body_id", "bullet"}, "bullet action");
    return SetBullet{
        id(member(value, "body_id", "action"), "body ID"),
        boolean(member(value, "bullet", "action"), "bullet")};
  }
  if (kind == "set_fixture_sensor") {
    require_members(value, {"kind", "fixture_id", "sensor"}, "fixture-sensor action");
    return SetFixtureSensor{
        id(member(value, "fixture_id", "action"), "fixture ID"),
        boolean(member(value, "sensor", "action"), "sensor")};
  }
  if (kind == "set_fixture_material") {
    require_members(
        value,
        {"kind", "fixture_id", "friction_bits", "restitution_bits"},
        "fixture-material action");
    const auto friction = u32(member(value, "friction_bits", "action"), "friction bits");
    const auto restitution = u32(member(value, "restitution_bits", "action"), "restitution bits");
    require_nonnegative(friction, "friction bits");
    require_nonnegative(restitution, "restitution bits");
    return SetFixtureMaterial{
        id(member(value, "fixture_id", "action"), "fixture ID"),
        friction,
        restitution};
  }
  if (kind == "set_fixture_filter") {
    require_members(value, {"kind", "fixture_id", "filter"}, "fixture-filter action");
    return SetFixtureFilter{
        id(member(value, "fixture_id", "action"), "fixture ID"),
        filter(member(value, "filter", "action"))};
  }
  if (kind == "set_fixture_density") {
    require_members(value, {"kind", "fixture_id", "density_bits"}, "fixture-density action");
    const auto density = u32(member(value, "density_bits", "action"), "density bits");
    require_nonnegative(density, "density bits");
    return SetFixtureDensity{
        id(member(value, "fixture_id", "action"), "fixture ID"), density};
  }
  if (kind == "reset_mass_data") {
    require_members(value, {"kind", "body_id"}, "reset-mass action");
    return ResetMassData{id(member(value, "body_id", "action"), "body ID")};
  }
  if (kind == "set_custom_mass_data") {
    require_members(
        value,
        {"kind", "body_id", "mass_bits", "center", "inertia_bits"},
        "custom-mass action");
    const auto mass = u32(member(value, "mass_bits", "action"), "mass bits");
    const auto inertia = u32(member(value, "inertia_bits", "action"), "inertia bits");
    const auto center = vec2(member(value, "center", "action"), "mass center");
    require_finite(mass, "mass bits");
    require_nonnegative(inertia, "inertia bits");
    const auto mass_value = float_from_bits(mass);
    if (mass_value <= 0.0F) {
      throw std::runtime_error("custom mass must be positive");
    }
    const auto origin_inertia = float_from_bits(inertia);
    if (origin_inertia > 0.0F) {
      const auto center_x = float_from_bits(center.x);
      const auto center_y = float_from_bits(center.y);
      const std::array<float, 2> squared_center{
          center_x * center_x, center_y * center_y};
      const auto center_dot = squared_center[0] + squared_center[1];
      const auto parallel_axis = mass_value * center_dot;
      const auto centered_inertia = origin_inertia - parallel_axis;
      if (!std::isfinite(squared_center[0]) ||
          !std::isfinite(squared_center[1]) || !std::isfinite(center_dot) ||
          !std::isfinite(parallel_axis) || !std::isfinite(centered_inertia) ||
          centered_inertia <= 0.0F) {
        throw std::runtime_error("custom mass centered inertia is invalid");
      }
    }
    return SetCustomMassData{
        id(member(value, "body_id", "action"), "body ID"),
        mass,
        center,
        inertia};
  }
  if (kind == "step") {
    require_members(
        value,
        {"kind", "timestep_bits", "velocity_iterations", "position_iterations"},
        "step action");
    const auto timestep = u32(member(value, "timestep_bits", "action"), "timestep bits");
    const auto velocity = u32(member(value, "velocity_iterations", "action"), "velocity iterations");
    const auto position = u32(member(value, "position_iterations", "action"), "position iterations");
    if (timestep != kRigidWorldTimestepBits ||
        velocity != kRigidWorldVelocityIterations ||
        position != kRigidWorldPositionIterations) {
      throw std::runtime_error("step action does not match the fixed Phase 6 tuple");
    }
    return RigidStep{timestep, velocity, position};
  }
  if (kind == "set_world_gravity") {
    require_members(value, {"kind", "gravity"}, "world-gravity action");
    return SetWorldGravity{vec2(member(value, "gravity", "action"), "gravity")};
  }
  if (kind == "set_automatic_force_clearing") {
    require_members(value, {"kind", "enabled"}, "automatic-force-clearing action");
    return SetAutomaticForceClearing{boolean(member(value, "enabled", "action"), "enabled")};
  }
  if (kind == "set_warm_starting") {
    require_members(value, {"kind", "enabled"}, "warm-starting action");
    return SetWarmStarting{boolean(member(value, "enabled", "action"), "enabled")};
  }
  if (kind == "set_continuous_physics") {
    require_members(value, {"kind", "enabled"}, "continuous-physics action");
    return SetContinuousPhysics{boolean(member(value, "enabled", "action"), "enabled")};
  }
  if (kind == "set_sub_stepping") {
    require_members(value, {"kind", "enabled"}, "sub-stepping action");
    return SetSubStepping{boolean(member(value, "enabled", "action"), "enabled")};
  }
  if (kind == "clear_forces") {
    require_members(value, {"kind"}, "clear-forces action");
    return ClearForces{};
  }
  if (kind == "configured_step") {
    require_members(
        value,
        {"kind", "timestep_bits", "velocity_iterations", "position_iterations", "continuous_work_budget"},
        "configured-step action");
    const auto timestep = u32(member(value, "timestep_bits", "action"), "timestep");
    const auto velocity = u32(member(value, "velocity_iterations", "action"), "velocity iterations");
    const auto position = u32(member(value, "position_iterations", "action"), "position iterations");
    const auto budget = u32(member(value, "continuous_work_budget", "action"), "continuous work budget");
    require_nonnegative(timestep, "timestep");
    if (velocity == 0 || velocity > kRigidWorldMaximumIterations ||
        position == 0 || position > kRigidWorldMaximumIterations ||
        budget == 0 || budget > kRigidWorldMaximumContinuousWork) {
      throw std::runtime_error("configured step is outside reviewed bounds");
    }
    return ConfiguredStep{timestep, velocity, position, budget};
  }
  if (kind == "query_aabb") {
    require_members(value, {"kind", "aabb", "directive_rules"}, "query action");
    const auto& raw_aabb = member(value, "aabb", "query action");
    require_members(raw_aabb, {"lower", "upper"}, "query AABB");
    const RigidAabbBits aabb{
        vec2(member(raw_aabb, "lower", "query AABB"), "lower bound"),
        vec2(member(raw_aabb, "upper", "query AABB"), "upper bound")};
    if (float_from_bits(aabb.lower.x) > float_from_bits(aabb.upper.x) ||
        float_from_bits(aabb.lower.y) > float_from_bits(aabb.upper.y)) {
      throw std::runtime_error("query AABB bounds are reversed");
    }
    return QueryAabb{aabb, query_rules(member(value, "directive_rules", "query action"))};
  }
  if (kind == "ray_cast") {
    require_members(value, {"kind", "start", "end", "directive_rules"}, "ray-cast action");
    const auto start = vec2(member(value, "start", "ray action"), "ray start");
    const auto end = vec2(member(value, "end", "ray action"), "ray end");
    if (start.x == end.x && start.y == end.y) {
      throw std::runtime_error("ray must have a non-zero direction");
    }
    return RayCast{start, end, ray_rules(member(value, "directive_rules", "ray action"))};
  }
  if (kind == "shift_origin") {
    require_members(value, {"kind", "shift"}, "origin-shift action");
    return ShiftOrigin{vec2(member(value, "shift", "action"), "origin shift")};
  }
  if (kind == "destroy_fixture") {
    require_members(value, {"kind", "fixture_id"}, "destroy-fixture action");
    return DestroyFixture{id(member(value, "fixture_id", "action"), "fixture ID")};
  }
  if (kind == "destroy_body") {
    require_members(value, {"kind", "body_id"}, "destroy-body action");
    return DestroyBody{id(member(value, "body_id", "action"), "body ID")};
  }
  throw std::runtime_error("unsupported rigid-world action");
}
