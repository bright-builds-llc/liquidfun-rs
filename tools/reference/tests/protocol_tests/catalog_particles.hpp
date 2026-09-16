// Catalog range actions must retain the pinned upstream distribution semantics.

nlohmann::json catalog_particle_range_payload(bool impulse) {
  auto payload = catalog_payload();
  payload["identity"]["slug"] = "particle-forces-and-statistics";
  payload["identity"]["settings"]["particle_iterations"] = 1U;
  payload["entities"] = nlohmann::json::array();
  payload["actions"] = nlohmann::json::array();
  for (std::uint32_t index = 0; index < 4U; ++index) {
    const auto id = index == 0U ? std::string("entity-particle-system-0000")
                               : "entity-particle-000" + std::to_string(index);
    const auto kind = index == 0U ? "particle_system" : "particle";
    payload["entities"].push_back({
        {"semantic_id", {{"kind", kind}, {"ordinal", index}}},
        {"scenario_id", id}});
    const nlohmann::json action = index == 0U
        ? nlohmann::json{{"kind", "create_system"}, {"system_id", id}}
        : nlohmann::json{{"kind", "create_particle"}, {"particle_id", id}};
    payload["actions"].push_back({
        {"action_id", "action-000" + std::to_string(index)},
        {"schedule", {{"kind", "setup"}, {"ordinal", index}}},
        {"action", {{"kind", "particle"}, {"action", action}}}});
  }
  const nlohmann::json vector = {
      {"x_bits", liquidfun::reference::bits_from_float(12.0F)},
      {"y_bits", liquidfun::reference::bits_from_float(-6.0F)}};
  const nlohmann::json action = {
      {"kind", impulse ? "apply_impulse" : "apply_force"},
      {"particle_ids", {"entity-particle-0001", "entity-particle-0002", "entity-particle-0003"}},
      {impulse ? "impulse" : "force", vector}};
  payload["actions"].push_back({
      {"action_id", "action-0004"},
      {"schedule", {{"kind", "logical_step"}, {"ordinal", 1U}}},
      {"action", {{"kind", "particle"}, {"action", action}}}});
  payload["actions"].push_back({
      {"action_id", "action-0005"},
      {"schedule", {{"kind", "logical_step"}, {"ordinal", 2U}}},
      {"action", {{"kind", "configured_step"},
                  {"timestep_bits", 0x3c888889U},
                  {"velocity_iterations", 8U}, {"position_iterations", 3U},
                  {"continuous_work_budget", 1U}}}});
  payload["checkpoints"] = {
      {{"checkpoint_id", "checkpoint-0001"}, {"after_action_id", "action-0004"},
       {"logical_step", 1U}},
      {{"checkpoint_id", "checkpoint-0002"}, {"after_action_id", "action-0005"},
       {"logical_step", 2U}}};
  return payload;
}

void catalog_particle_range_matches_upstream_after_step(bool impulse) {
  // Arrange
  b2World world({0.0F, -10.0F});
  b2ParticleSystemDef definition;
  auto* system = world.CreateParticleSystem(&definition);
  for (int32 index = 0; index < 3; ++index) {
    system->CreateParticle(b2ParticleDef{});
  }
  const b2Vec2 vector(12.0F, -6.0F);
  if (impulse) {
    system->ApplyLinearImpulse(0, 3, vector);
  } else {
    system->ApplyForce(0, 3, vector);
  }
  world.Step(1.0F / 60.0F, 8, 3, 1);
  const auto identity = std::string(64, 'a');
  liquidfun::reference::CatalogRunAdapter adapter;

  // Act
  const auto trace = adapter.execute(
      catalog_request_from_payload(catalog_particle_range_payload(impulse), identity),
      identity);
  const auto checkpoint = nlohmann::json::parse(trace.checkpoint_records.at(1));

  // Assert
  const auto& primitives = checkpoint.at("debug_primitives");
  expect(primitives.size() == 3U, "particle capture omitted upstream geometry");
  for (std::size_t index = 0; index < 3U; ++index) {
    const auto& position = system->GetPositionBuffer()[index];
    const auto& center = primitives.at(index).at("primitive").at("value").at("center");
    expect(center.at("x_bits") == liquidfun::reference::bits_from_float(position.x) &&
               center.at("y_bits") == liquidfun::reference::bits_from_float(position.y),
           impulse ? "catalog impulse differs from upstream range distribution"
                   : "catalog force differs from upstream range distribution");
  }
}

void catalog_particle_capture_preserves_geometry_and_color(bool colored) {
  // Arrange
  using liquidfun::reference::catalog_run_detail::CatalogParticle;
  using liquidfun::reference::catalog_run_detail::CatalogParticleDraw;
  b2World world({0.0F, -10.0F});
  b2ParticleSystemDef definition;
  definition.radius = 0.5F;
  auto* system = world.CreateParticleSystem(&definition);
  b2ParticleDef particle;
  particle.position.Set(2.0F, -3.0F);
  if (colored) {
    particle.color = b2ParticleColor(10, 20, 30, 40);
  }
  const auto index = system->CreateParticle(particle);
  const std::vector<std::pair<std::string, CatalogParticle>> identities = {
      {"particle-stable", {system, system->GetParticleHandleFromIndex(index)}}};
  CatalogParticleDraw draw(identities);
  world.SetDebugDraw(&draw);

  // Act
  world.DrawDebugData();
  world.SetDebugDraw(nullptr);

  // Assert
  expect(draw.count == 1U && draw.primitives.size() == 1U,
         "particle capture count differs from serialized geometry");
  const auto& value = draw.primitives.at(0).at("primitive").at("value");
  expect(value.at("center").at("x_bits") == liquidfun::reference::bits_from_float(2.0F) &&
             value.at("center").at("y_bits") == liquidfun::reference::bits_from_float(-3.0F) &&
             value.at("radius_bits") == liquidfun::reference::bits_from_float(0.5F),
         "particle capture did not use actual upstream geometry");
  const auto& metadata = value.at("metadata");
  expect(metadata.at("key").at("owner").at("semantic_id") == "particle-stable",
         "particle capture lost stable ownership");
  const nlohmann::json expected_color = colored
      ? nlohmann::json{10U, 20U, 30U, 40U}
      : nlohmann::json{57U, 197U, 207U, 220U};
  expect(metadata.at("stroke").at("color") == expected_color &&
             metadata.at("maybe_fill").at("color") == expected_color,
         "particle capture changed source or default color");
}

void catalog_particle_recreation_retires_destroyed_identity(bool paused, bool group_step) {
  // Arrange
  auto payload = catalog_particle_range_payload(false);
  auto& actions = payload["actions"];
  actions.erase(actions.begin() + 3, actions.end());
  actions.push_back({
      {"action_id", "action-0003"},
      {"schedule", {{"kind", "setup"}, {"ordinal", 3U}}},
      {"action", {{"kind", "particle"}, {"action", {
          {"kind", "set_paused"}, {"system_id", "entity-particle-system-0000"},
          {"paused", paused}}}}}});
  auto step = catalog_particle_range_payload(false)["actions"][5]["action"];
  if (group_step) {
    auto operation = step;
    operation["kind"] = "step";
    operation.erase("continuous_work_budget");
    operation["particle_iterations"] = 1U;
    step = {{"kind", "particle_group"}, {"operation", operation}};
  }
  std::vector<nlohmann::json> logical = {
      {{"kind", "particle"}, {"action", {{"kind", "mark_for_destruction"},
          {"particle_id", "entity-particle-0001"}}}},
      {{"kind", "particle"}, {"action", {{"kind", "compact"},
          {"system_id", "entity-particle-system-0000"}}}},
      step,
      {{"kind", "particle"}, {"action", {{"kind", "create_particle"},
          {"particle_id", "entity-particle-0003"}}}}};
  if (group_step) {
    logical[1] = step;
    logical[1]["operation"]["timestep_bits"] = 0U;
  }
  payload["checkpoints"] = nlohmann::json::array();
  for (std::size_t index = 0; index < logical.size(); ++index) {
    const auto action_id = "action-000" + std::to_string(index + 4U);
    actions.push_back({
        {"action_id", action_id},
        {"schedule", {{"kind", "logical_step"}, {"ordinal", index + 1U}}},
        {"action", logical[index]}});
    payload["checkpoints"].push_back({
        {"checkpoint_id", "checkpoint-000" + std::to_string(index + 1U)},
        {"after_action_id", action_id}, {"logical_step", index + 1U}});
  }
  const auto identity = std::string(64, 'a');
  liquidfun::reference::CatalogRunAdapter adapter;

  // Act
  const auto trace = adapter.execute(catalog_request_from_payload(payload, identity), identity);

  // Assert
  const auto pending = nlohmann::json::parse(trace.checkpoint_records.at(1));
  expect(pending.at("debug_primitives").size() == 2U,
         "zero timestep prematurely retired a pending particle");
  const auto compacted = nlohmann::json::parse(trace.checkpoint_records.at(2));
  expect(compacted.at("debug_primitives").size() == 1U,
         "positive timestep failed to compact a pending particle");
  const auto recreated = nlohmann::json::parse(trace.checkpoint_records.at(3));
  const auto& primitives = recreated.at("debug_primitives");
  expect(primitives.size() == 2U, "recreated particle was not captured");
  for (std::size_t index = 0; index < 2U; ++index) {
    const auto& owner = primitives.at(index).at("primitive").at("value")
        .at("metadata").at("key").at("owner").at("semantic_id");
    expect(owner == "entity-particle-000" + std::to_string(index + 2U),
           "recreated particle inherited a destroyed semantic identity");
  }
}
