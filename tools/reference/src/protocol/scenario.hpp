ScenarioSource decode_source(const Node& node) {
  const auto& object = as_object(node, "scenario source");
  const auto& kind = as_string(member(object, "kind", "scenario source"), "source kind");
  if (kind == "named") {
    require_members(object, {"kind", "name"}, "named source");
    const auto& name = as_string(member(object, "name", "named source"), "source name");
    if (name.find_first_not_of(" \t\r\n") == std::string::npos) {
      throw std::runtime_error("named source must not be blank");
    }
    return ScenarioSource{ScenarioSourceKind::named, name, {}, 0, 0};
  }
  if (kind == "seeded") {
    require_members(
        object,
        {"kind", "generator_id", "generator_version", "seed"},
        "seeded source");
    const auto& generator_id = as_string(
        member(object, "generator_id", "seeded source"), "generator ID");
    const auto generator_version = as_u32(
        member(object, "generator_version", "seeded source"),
        "generator version");
    if (generator_id.find_first_not_of(" \t\r\n") == std::string::npos ||
        generator_version == 0) {
      throw std::runtime_error("seeded source is invalid");
    }
    return ScenarioSource{
        ScenarioSourceKind::seeded,
        {},
        generator_id,
        generator_version,
        as_u64(member(object, "seed", "seeded source"), "seed")};
  }
  throw std::runtime_error("unsupported source kind");
}

StepCommand decode_command(const Node& node) {
  const auto& object = as_object(node, "step command");
  require_members(
      object,
      {"kind", "command_id", "timestep_bits", "velocity_iterations",
       "position_iterations", "particle_iterations"},
      "step command");
  if (as_string(member(object, "kind", "step command"), "command kind") != "step") {
    throw std::runtime_error("unsupported command kind");
  }
  StepCommand command{
      as_string(member(object, "command_id", "step command"), "command ID"),
      as_u32(member(object, "timestep_bits", "step command"), "timestep bits"),
      as_u32(member(object, "velocity_iterations", "step command"), "velocity iterations"),
      as_u32(member(object, "position_iterations", "step command"), "position iterations"),
      as_u32(member(object, "particle_iterations", "step command"), "particle iterations")};
  require_id(command.command_id, "command ID");
  for (const auto iterations : {command.velocity_iterations,
                                command.position_iterations,
                                command.particle_iterations}) {
    if (iterations == 0 || iterations > 255) {
      throw std::runtime_error("solver iterations are outside reviewed bounds");
    }
  }
  return command;
}

CheckpointRequest decode_checkpoint(const Node& node) {
  const auto& object = as_object(node, "checkpoint");
  require_members(
      object,
      {"checkpoint_id", "after_command_id", "phase", "observables"},
      "checkpoint");
  CheckpointRequest checkpoint{
      as_string(member(object, "checkpoint_id", "checkpoint"), "checkpoint ID"),
      as_string(member(object, "after_command_id", "checkpoint"), "command reference"),
      as_string(member(object, "phase", "checkpoint"), "checkpoint phase"),
      {}};
  require_id(checkpoint.checkpoint_id, "checkpoint ID");
  require_id(checkpoint.after_command_id, "checkpoint command reference");
  if (checkpoint.phase.empty()) {
    throw std::runtime_error("checkpoint phase must not be empty");
  }
  const auto& observables = as_array(
      member(object, "observables", "checkpoint"), "checkpoint observables");
  if (observables.size() > kMaximumObservableItems) {
    throw std::runtime_error("observable collection exceeds reviewed limit");
  }
  std::set<Observable> unique;
  for (const auto& observable_node : observables) {
    const auto& value = as_string(observable_node, "observable");
    const auto observable = value == "world_counts"
                                ? Observable::world_counts
                                : value == "simulation_time"
                                      ? Observable::simulation_time
                                      : throw std::runtime_error("unsupported observable");
    if (!unique.insert(observable).second) {
      throw std::runtime_error("duplicate observable");
    }
    checkpoint.observables.push_back(observable);
  }
  return checkpoint;
}

ScenarioV1 decode_scenario(const Node& node) {
  const auto& object = as_object(node, "scenario");
  require_members(
      object,
      {"scenario_id", "source", "gravity_x_bits", "gravity_y_bits", "entities",
       "commands", "checkpoints"},
      "scenario");
  ScenarioV1 scenario{
      as_string(member(object, "scenario_id", "scenario"), "scenario ID"),
      decode_source(member(object, "source", "scenario")),
      as_u32(member(object, "gravity_x_bits", "scenario"), "gravity x bits"),
      as_u32(member(object, "gravity_y_bits", "scenario"), "gravity y bits"),
      {},
      {}};
  require_id(scenario.scenario_id, "scenario ID");
  if (!as_array(member(object, "entities", "scenario"), "entities").empty()) {
    throw std::runtime_error("phase-2 entities must be empty");
  }
  const auto& commands = as_array(member(object, "commands", "scenario"), "commands");
  if (commands.empty()) {
    throw std::runtime_error("scenario must contain a command");
  }
  std::unordered_set<std::string> command_ids;
  for (const auto& command_node : commands) {
    auto command = decode_command(command_node);
    if (!command_ids.insert(command.command_id).second) {
      throw std::runtime_error("duplicate command ID");
    }
    scenario.commands.push_back(std::move(command));
  }
  const auto& checkpoints = as_array(
      member(object, "checkpoints", "scenario"), "checkpoints");
  std::unordered_set<std::string> checkpoint_ids;
  std::size_t previous_command_index = 0;
  for (const auto& checkpoint_node : checkpoints) {
    auto checkpoint = decode_checkpoint(checkpoint_node);
    if (!checkpoint_ids.insert(checkpoint.checkpoint_id).second) {
      throw std::runtime_error("duplicate checkpoint ID");
    }
    const auto command = std::find_if(
        scenario.commands.begin(), scenario.commands.end(),
        [&checkpoint](const auto& candidate) {
          return candidate.command_id == checkpoint.after_command_id;
        });
    if (command == scenario.commands.end()) {
      throw std::runtime_error("checkpoint command reference is unknown");
    }
    const auto command_index = static_cast<std::size_t>(
        std::distance(scenario.commands.begin(), command));
    if (!scenario.checkpoints.empty() && command_index < previous_command_index) {
      throw std::runtime_error("checkpoint command references are out of order");
    }
    previous_command_index = command_index;
    scenario.checkpoints.push_back(std::move(checkpoint));
  }
  return scenario;
}

std::string encode_source(const ScenarioSource& source) {
  if (source.kind == ScenarioSourceKind::named) {
    return "{\"kind\":\"named\",\"name\":" + quote(source.name) + "}";
  }
  return "{\"kind\":\"seeded\",\"generator_id\":" +
         quote(source.generator_id) + ",\"generator_version\":" +
         std::to_string(source.generator_version) + ",\"seed\":" +
         std::to_string(source.seed) + "}";
}

std::string encode_world_counts(const WorldCounts& counts) {
  return "{\"bodies\":" + std::to_string(counts.bodies) +
         ",\"fixtures\":" + std::to_string(counts.fixtures) +
         ",\"joints\":" + std::to_string(counts.joints) +
         ",\"contacts\":" + std::to_string(counts.contacts) +
         ",\"particle_systems\":" + std::to_string(counts.particle_systems) +
         ",\"particle_groups\":" + std::to_string(counts.particle_groups) +
         ",\"particles\":" + std::to_string(counts.particles) + "}";
}
