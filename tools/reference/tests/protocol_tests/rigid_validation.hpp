void rigid_world_rejects_untrusted_records_before_execution() {
  // Arrange
  const auto fixture = read_fixture(
      "protocol/fixtures/accepted/rigid-world-request.jsonl");
  auto duplicate = fixture;
  duplicate.insert(1, "\"protocol_version\":1,");
  auto unknown = fixture;
  unknown.insert(1, "\"unexpected\":true,");
  auto out_of_order_json = nlohmann::json::parse(fixture);
  auto& out_of_order_actions =
      out_of_order_json.at("scenario").at("timelines").at(0).at("actions");
  std::swap(out_of_order_actions.at(0), out_of_order_actions.at(3));
  const auto out_of_order = out_of_order_json.dump() + '\n';
  auto oversized = nlohmann::json::parse(fixture);
  auto& actions = oversized.at("scenario").at("timelines").at(0).at("actions");
  while (actions.size() <= liquidfun::reference::kRigidWorldMaximumActions) {
    actions.push_back(actions.back());
  }
  const auto oversized_record = oversized.dump() + '\n';
  auto missing_static_kinematic = fixture;
  const auto static_kinematic =
      missing_static_kinematic.find("static_kinematic_overlap_rejected");
  expect(
      static_kinematic != std::string::npos,
      "static/kinematic admission witness is missing from fixture");
  missing_static_kinematic.replace(
      static_kinematic,
      std::string("static_kinematic_overlap_rejected").size(),
      "removed_static_kinematic_witness");
  auto missing_kinematic_kinematic = fixture;
  const auto kinematic_kinematic =
      missing_kinematic_kinematic.find("kinematic_kinematic_overlap_rejected");
  expect(
      kinematic_kinematic != std::string::npos,
      "kinematic/kinematic admission witness is missing from fixture");
  missing_kinematic_kinematic.replace(
      kinematic_kinematic,
      std::string("kinematic_kinematic_overlap_rejected").size(),
      "removed_kinematic_kinematic_witness");

  // Act / Assert
  for (const auto& [record, expected] :
       std::vector<std::pair<std::string, std::string>>{
           {duplicate, "duplicate member"},
           {unknown, "unknown member"},
           {out_of_order, "action order"},
           {oversized_record, "action count"},
           {missing_static_kinematic, "witness registry is incomplete"},
           {missing_kinematic_kinematic, "witness registry is incomplete"}}) {
    try {
      static_cast<void>(decode_rigid_world_request(record));
    } catch (const std::exception& error) {
      expect(
          std::string(error.what()).find(expected) != std::string::npos,
          "unexpected rigid rejection: " + std::string(error.what()));
      continue;
    }
    throw std::runtime_error("untrusted rigid record was accepted");
  }
}

void rigid_world_boundary_matches_the_fixed_rust_contract() {
  // Arrange
  const auto fixture = read_fixture(
      "protocol/fixtures/accepted/rigid-world-request.jsonl");
  auto maximum = nlohmann::json::parse(fixture);
  auto& maximum_actions =
      maximum.at("scenario").at("timelines").at(0).at("actions");
  const auto inspect_template = maximum_actions.at(9);
  while (maximum_actions.size() <
         liquidfun::reference::kRigidWorldMaximumActions) {
    auto action = inspect_template;
    action["action_id"] =
        "maximum-action-" + std::to_string(maximum_actions.size());
    maximum_actions.insert(maximum_actions.end() - 6, std::move(action));
  }
  auto maximum_plus_one = maximum;
  auto& maximum_plus_one_actions =
      maximum_plus_one.at("scenario").at("timelines").at(0).at("actions");
  auto extra = inspect_template;
  extra["action_id"] = "maximum-action-128";
  maximum_plus_one_actions.insert(
      maximum_plus_one_actions.end() - 6, std::move(extra));

  std::vector<nlohmann::json> alternate_steps;
  for (const auto& [field, value] :
       std::vector<std::pair<std::string, std::uint32_t>>{
           {"timestep_bits", liquidfun::reference::kRigidWorldTimestepBits + 1},
           {"velocity_iterations",
            liquidfun::reference::kRigidWorldVelocityIterations + 1},
           {"position_iterations",
            liquidfun::reference::kRigidWorldPositionIterations + 1}}) {
    auto alternate = nlohmann::json::parse(fixture);
    auto& timeline_actions =
        alternate.at("scenario").at("timelines").at(0).at("actions");
    auto step = std::find_if(
        timeline_actions.begin(), timeline_actions.end(),
        [](const auto& action) {
          return action.at("action_id") == "nc-step-zero";
        });
    expect(step != timeline_actions.end(), "fixed step action is missing");
    step->at("action")[field] = value;
    alternate_steps.push_back(std::move(alternate));
  }

  auto invalid_mass = nlohmann::json::parse(fixture);
  auto& invalid_mass_actions =
      invalid_mass.at("scenario").at("timelines").at(0).at("actions");
  auto custom_mass = std::find_if(
      invalid_mass_actions.begin(), invalid_mass_actions.end(),
      [](const auto& action) {
        return action.at("action_id") == "nc-custom-mass";
      });
  expect(custom_mass != invalid_mass_actions.end(), "custom mass action is missing");
  custom_mass->at("action")["mass_bits"] = 0x3f800000U;
  custom_mass->at("action")["center"]["x_bits"] = 0x40000000U;
  custom_mass->at("action")["center"]["y_bits"] = 0U;
  custom_mass->at("action")["inertia_bits"] = 0x3f800000U;

  // Act
  const auto accepted = decode_rigid_world_request(maximum.dump() + '\n');

  // Assert
  expect(
      accepted.timelines.at(0).actions.size() ==
          liquidfun::reference::kRigidWorldMaximumActions,
      "exact rigid action maximum was rejected");
  for (const auto& [record, expected] :
       std::vector<std::pair<std::string, std::string>>{
           {maximum_plus_one.dump() + '\n', "action count"},
           {alternate_steps.at(0).dump() + '\n', "fixed Phase 6 tuple"},
           {alternate_steps.at(1).dump() + '\n', "fixed Phase 6 tuple"},
           {alternate_steps.at(2).dump() + '\n', "fixed Phase 6 tuple"},
           {invalid_mass.dump() + '\n', "centered inertia"}}) {
    try {
      static_cast<void>(decode_rigid_world_request(record));
    } catch (const std::exception& error) {
      expect(
          std::string(error.what()).find(expected) != std::string::npos,
          "unexpected rigid boundary rejection: " + std::string(error.what()));
      continue;
    }
    throw std::runtime_error("invalid rigid boundary record was accepted");
  }
}

void rigid_world_rejects_zero_centered_inertia_before_execution() {
  // Arrange
  const auto fixture = read_fixture(
      "protocol/fixtures/rejected/rigid-world-zero-centered-inertia.jsonl");

  // Act / Assert
  try {
    static_cast<void>(decode_rigid_world_request(fixture));
  } catch (const std::exception& error) {
    expect(
        std::string(error.what()).find("centered inertia") != std::string::npos,
        "zero centered inertia produced an unstable diagnostic");
    return;
  }
  throw std::runtime_error("zero centered inertia reached adapter execution");
}

void rigid_world_accepts_zero_origin_inertia_with_nonzero_center() {
  // Arrange
  auto request = nlohmann::json::parse(read_fixture(
      "protocol/fixtures/accepted/rigid-world-request.jsonl"));
  auto& action = custom_mass_action(request);
  action["mass_bits"] = 0x3f800000U;
  action["center"]["x_bits"] = 0x3f800000U;
  action["center"]["y_bits"] = 0U;
  action["inertia_bits"] = 0U;

  // Act
  const auto decoded = decode_rigid_world_request(request.dump() + '\n');

  // Assert
  expect(
      decoded.timelines.at(0).actions.size() ==
          request.at("scenario").at("timelines").at(0).at("actions").size(),
      "zero origin inertia did not preserve the reviewed action timeline");
}

void rigid_world_rejects_non_finite_centered_inertia_intermediates() {
  // Arrange
  auto request = nlohmann::json::parse(read_fixture(
      "protocol/fixtures/accepted/rigid-world-request.jsonl"));
  auto& action = custom_mass_action(request);
  action["mass_bits"] = 0x3f800000U;
  action["center"]["x_bits"] = 0x7f7fffffU;
  action["center"]["y_bits"] = 0U;
  action["inertia_bits"] = 0x7f7fffffU;

  // Act / Assert
  try {
    static_cast<void>(decode_rigid_world_request(request.dump() + '\n'));
  } catch (const std::exception& error) {
    expect(
        std::string(error.what()).find("centered inertia") != std::string::npos,
        "non-finite centered inertia produced an unstable diagnostic");
    return;
  }
  throw std::runtime_error("non-finite centered inertia was accepted");
}

void rigid_world_reuse_advances_reset_without_state_leakage() {
  // Arrange
  const auto fixture = read_fixture(
      "protocol/fixtures/accepted/rigid-world-request.jsonl");
  RigidWorldAdapter adapter;

  // Act
  const auto first = adapter.execute(fixture);
  const auto second = adapter.execute(fixture);
  const auto third = adapter.execute(fixture);

  // Assert
  expect(first.reset_epoch == 1, "first rigid reset epoch changed");
  expect(second.reset_epoch == 2, "second rigid reset epoch changed");
  expect(third.reset_epoch == 3, "third rigid reset epoch changed");
  expect(first.result_record == second.result_record, "rigid request leaked state");
  expect(second.result_record == third.result_record, "rigid allocation history changed identity");
}

void rigid_world_phase8_decode_fails_closed_at_reviewed_boundaries() {
  // Arrange
  constexpr std::size_t maximum_phase8_ropes = 8;
  constexpr std::size_t maximum_phase8_rope_vertices = 128;
  const auto fixture = read_fixture(
      "protocol/fixtures/accepted/rigid-world-request.jsonl");
  auto unknown_action = nlohmann::json::parse(fixture);
  unknown_action["scenario"]["timelines"][9]["actions"][0]["action"]["kind"] =
      "unknown_phase8_action";
  auto too_many_joints = nlohmann::json::parse(fixture);
  auto& joints = too_many_joints["scenario"]["timelines"][9]["joints"];
  while (joints.size() <= liquidfun::reference::kRigidWorldMaximumJoints) {
    auto duplicate = joints.at(0);
    duplicate["joint_id"] = "bounded-joint-" + std::to_string(joints.size());
    joints.push_back(std::move(duplicate));
  }
  auto missing_family = nlohmann::json::parse(fixture);
  missing_family["scenario"]["timelines"].erase(
      missing_family["scenario"]["timelines"].begin() + 9);
  auto zero_step = nlohmann::json::parse(fixture);
  zero_step["scenario"]["timelines"][9]["actions"][16]["action"]
           ["timestep_bits"] = 0U;
  auto missing_post_step_observation = nlohmann::json::parse(fixture);
  auto& observation_actions =
      missing_post_step_observation["scenario"]["timelines"][10]["actions"];
  const auto missing_observation = std::find_if(
      observation_actions.begin(), observation_actions.end(), [](const auto& record) {
        return record.at("action_id") ==
               "joint-rp-inspect-joint-rp-revolute-cold";
      });
  expect(
      missing_observation != observation_actions.end(),
      "reviewed Phase 8 observation action is missing from the fixture");
  observation_actions.erase(missing_observation);
  auto duplicate_id = nlohmann::json::parse(fixture);
  duplicate_id["scenario"]["timelines"][9]["bodies"][1]["body_id"] =
      duplicate_id["scenario"]["timelines"][9]["bodies"][0]["body_id"];
  auto too_many_actions = nlohmann::json::parse(fixture);
  auto& actions = too_many_actions["scenario"]["timelines"][9]["actions"];
  while (actions.size() <= liquidfun::reference::kRigidWorldMaximumActions) {
    auto duplicate = actions.back();
    duplicate["action_id"] = "bounded-action-" + std::to_string(actions.size());
    actions.push_back(std::move(duplicate));
  }
  auto too_many_ropes = nlohmann::json::parse(fixture);
  auto& ropes = too_many_ropes["scenario"]["timelines"][15]["ropes"];
  while (ropes.size() <= maximum_phase8_ropes) {
    auto duplicate = ropes.at(0);
    duplicate["rope_id"] = "bounded-rope-" + std::to_string(ropes.size());
    ropes.push_back(std::move(duplicate));
  }
  auto too_many_rope_vertices = nlohmann::json::parse(fixture);
  auto& bounded_rope =
      too_many_rope_vertices["scenario"]["timelines"][15]["ropes"][0];
  while (bounded_rope["vertices"].size() <= maximum_phase8_rope_vertices) {
    bounded_rope["vertices"].push_back(bounded_rope["vertices"].back());
    bounded_rope["masses_bits"].push_back(bounded_rope["masses_bits"].back());
  }

  // Act / Assert
  for (const auto& [request, expected] :
       std::array<std::pair<nlohmann::json, std::string_view>, 9>{
           std::pair{unknown_action, "unsupported Phase 8 action kind"},
           std::pair{too_many_joints, "collection count"},
           std::pair{missing_family, "timeline count"},
           std::pair{zero_step, "must be positive"},
           std::pair{missing_post_step_observation, "post-step observation"},
           std::pair{duplicate_id, "duplicate Phase 8 ID"},
           std::pair{too_many_actions, "collection count"},
           std::pair{too_many_ropes, "collection count"},
           std::pair{too_many_rope_vertices, "rope vertex count"}}) {
    try {
      static_cast<void>(decode_rigid_world_request(request.dump() + '\n'));
    } catch (const std::exception& error) {
      expect(
          std::string(error.what()).find(expected) != std::string::npos,
          "Phase 8 boundary expected diagnostic containing '" +
              std::string(expected) + "' but received '" + error.what() + "'");
      continue;
    }
    throw std::runtime_error("invalid Phase 8 request was accepted");
  }
}

void phase8_reactions_guard_uninitialized_solver_scratch() {
  // Arrange / Act / Assert
  expect(
      liquidfun::reference::phase8_reaction_guard_self_test(),
      "Phase 8 reaction guard did not separate undefined and initialized state");
}
