void rigid_world_executes_all_complete_witness_families() {
  // Arrange
  const auto fixture = read_fixture(
      "protocol/fixtures/accepted/rigid-world-request.jsonl");
  RigidWorldAdapter adapter;

  // Act
  const auto trace = adapter.execute(fixture);
  const auto result = nlohmann::json::parse(trace.result_record);

  // Assert
  expect(trace.reset_verified, "rigid-world reset was not verified");
  expect(trace.reset_epoch == 1, "first rigid-world reset epoch was not one");
  expect(
      liquidfun::reference::decode_request_kind(fixture) ==
          liquidfun::reference::RequestKind::rigid_world,
      "rigid-world request did not use the existing process loop");
  expect(
      result.at("record_kind") == "rigid_world_result",
      "rigid-world result kind changed");
  const std::array expected_families{
      "non_colliding_body_fixture_lifecycle",
      "single_contact_lifecycle",
      "body_control_and_force_policy",
      "multi_contact_island_and_warm_start",
      "sleeping_and_waking",
      "continuous_collision_and_sub_stepping",
      "continuous_budget_resume",
      "world_query_and_ray_cast",
      "origin_shift_covariance",
      "joint_definitions_and_mutations",
      "revolute_prismatic_limits_and_motors",
      "distance_pulley_mouse_constraints",
      "wheel_weld_friction_rope_motor_constraints",
      "gear_dependencies_and_four_body_solver",
      "mixed_joint_island_order_and_collision_suppression",
      "standalone_rope_evolution",
      "contact_filter_listener_and_pre_solve_timing",
      "destruction_listener_and_dependency_cascades",
      "diagnostic_reconstruction_and_dump_order"};
  const std::array expected_phase7_checkpoints{
      "control-checkpoint",
      "island-checkpoint",
      "sleep-checkpoint",
      "ccd-checkpoint",
      "budget-checkpoint",
      "query-checkpoint",
      "origin-checkpoint"};
  const auto& timelines = result.at("timelines");
  expect(
      timelines.size() == expected_families.size(),
      "rigid witness families are incomplete");
  for (std::size_t index = 0; index < expected_families.size(); ++index) {
    expect(
        timelines.at(index).at("witness_family") == expected_families[index],
        "rigid witness family order changed");
  }
  for (std::size_t index = 0; index < expected_phase7_checkpoints.size(); ++index) {
    const auto& checkpoints = timelines.at(index + 2).at("checkpoints");
    expect(
        checkpoints.size() == 1 &&
            checkpoints.at(0).at("checkpoint_id") ==
                expected_phase7_checkpoints[index],
        "Phase 7 rigid checkpoint coverage is incomplete");
  }
  const auto& joint_observations =
      timelines.at(9).at("checkpoints").at(0).at("observations");
  expect(joint_observations.size() == 23, "Phase 8 joint coverage is incomplete");
  expect(
      joint_observations.at(10).at("snapshot").at("dependencies").size() == 2,
      "Phase 8 gear dependencies are incomplete");
  std::set<std::string> stepped_joint_kinds;
  for (std::size_t index = 12; index < joint_observations.size(); ++index) {
    stepped_joint_kinds.insert(
        joint_observations.at(index).at("snapshot").at("joint_kind"));
  }
  expect(
      stepped_joint_kinds.size() == 11,
      "Phase 8 did not inspect every pinned joint kind after stepping");
  const auto nontrivial_joint_count = std::count_if(
          joint_observations.begin() + 12,
          joint_observations.end(),
          [](const auto& observation) {
            const auto& snapshot = observation.at("snapshot");
            return snapshot.at("branch_state") != "inactive" ||
                   snapshot.at("coordinate_bits") != 0U ||
                   snapshot.at("speed_bits") != 0U ||
                   snapshot.at("reaction_force").at("x_bits") != 0U ||
                   snapshot.at("reaction_force").at("y_bits") != 0U ||
                   snapshot.at("reaction_torque_bits") != 0U;
          });
  expect(
      nontrivial_joint_count >= 10,
      "Phase 8 post-step joint observations remained trivial");
  const auto& rope_observations =
      timelines.at(15).at("checkpoints").at(0).at("observations");
  expect(rope_observations.size() == 3, "Phase 8 rope coverage is incomplete");
  std::set<std::string> gear_ids;
  for (const auto& observation :
       timelines.at(13).at("checkpoints").at(0).at("observations")) {
    if (observation.at("kind") == "joint" &&
        observation.at("snapshot").at("joint_kind") == "gear") {
      gear_ids.insert(observation.at("snapshot").at("joint_id"));
    }
  }
  expect(
      gear_ids == std::set<std::string>{
                      "gear-0-joint", "gear-1-joint", "gear-2-joint",
                      "gear-3-joint"},
      "Phase 8 did not execute all four gear source combinations");
  const auto& diagnostic_observations =
      timelines.at(18).at("checkpoints").at(0).at("observations");
  expect(
      diagnostic_observations.size() == 17 &&
          diagnostic_observations.back().at("kind") == "diagnostics" &&
          std::any_of(
              diagnostic_observations.begin(),
              diagnostic_observations.end(),
              [](const auto& observation) {
                return observation.at("kind") == "reconstruction" &&
                       observation.at("record").at("support") ==
                           "unsupported_mouse_joint";
              }),
      "Phase 8 reconstruction or diagnostics are incomplete");
  const auto& callback_lifecycle =
      timelines.at(16).at("checkpoints").at(0).at("observations");
  expect(
      callback_lifecycle == nlohmann::json::parse(
          R"([{"kind":"lifecycle","event":{"ordinal":0,"kind":"filter_decision","maybe_contact":null,"maybe_entity_id":"callback-fa"}},{"kind":"lifecycle","event":{"ordinal":1,"kind":"filter_decision","maybe_contact":null,"maybe_entity_id":"callback-fa"}},{"kind":"lifecycle","event":{"ordinal":2,"kind":"contact_created","maybe_contact":{"fixture_a_id":"callback-fa","child_a":0,"fixture_b_id":"callback-fb","child_b":0,"occurrence":1},"maybe_entity_id":null}},{"kind":"lifecycle","event":{"ordinal":3,"kind":"begin_contact","maybe_contact":{"fixture_a_id":"callback-fa","child_a":0,"fixture_b_id":"callback-fb","child_b":0,"occurrence":1},"maybe_entity_id":null}},{"kind":"lifecycle","event":{"ordinal":4,"kind":"pre_solve","maybe_contact":{"fixture_a_id":"callback-fa","child_a":0,"fixture_b_id":"callback-fb","child_b":0,"occurrence":1},"maybe_entity_id":null}},{"kind":"lifecycle","event":{"ordinal":5,"kind":"post_solve","maybe_contact":{"fixture_a_id":"callback-fa","child_a":0,"fixture_b_id":"callback-fb","child_b":0,"occurrence":1},"maybe_entity_id":null}},{"kind":"lifecycle","event":{"ordinal":6,"kind":"pre_solve","maybe_contact":{"fixture_a_id":"callback-fa","child_a":0,"fixture_b_id":"callback-fb","child_b":0,"occurrence":1},"maybe_entity_id":null}},{"kind":"lifecycle","event":{"ordinal":7,"kind":"post_solve","maybe_contact":{"fixture_a_id":"callback-fa","child_a":0,"fixture_b_id":"callback-fb","child_b":0,"occurrence":1},"maybe_entity_id":null}},{"kind":"lifecycle","event":{"ordinal":8,"kind":"pre_solve","maybe_contact":{"fixture_a_id":"callback-fa","child_a":0,"fixture_b_id":"callback-fb","child_b":0,"occurrence":1},"maybe_entity_id":null}}])"),
      "Phase 8 callback lifecycle order or multiplicity changed");
  const auto& destruction_observations =
      timelines.at(17).at("checkpoints").at(0).at("observations");
  std::vector<std::string> destruction_kinds;
  for (const auto& observation : destruction_observations) {
    if (observation.at("kind") == "lifecycle") {
      destruction_kinds.push_back(observation.at("event").at("kind"));
    }
  }
  expect(
      destruction_kinds == std::vector<std::string>{
          "filter_decision", "filter_decision", "contact_created",
          "begin_contact", "pre_solve", "post_solve", "pre_solve",
          "post_solve", "joint_goodbye", "end_contact",
          "contact_destroyed", "fixture_goodbye", "body_destroyed"},
      "Phase 8 destruction lifecycle order or multiplicity changed");
  const auto& non_colliding = result.at("timelines").at(0).at("checkpoints");
  const auto& single_contact = result.at("timelines").at(1).at("checkpoints");
  expect(non_colliding.size() == 8, "non-colliding checkpoints are incomplete");
  expect(single_contact.size() == 10, "contact checkpoints are incomplete");
  expect(
      non_colliding.at(1).at("checkpoint_id") ==
              "nc-static-kinematic-rejected" &&
          non_colliding.at(1).at("counts").at("contacts") == 0 &&
          non_colliding.at(1).at("counts").at("manifold_points") == 0 &&
          non_colliding.at(1).at("counts").at("events") == 0,
      "static/kinematic admission checkpoint changed");
  expect(
      non_colliding.at(3).at("checkpoint_id") ==
              "nc-kinematic-kinematic-rejected" &&
          non_colliding.at(3).at("counts").at("contacts") == 0 &&
          non_colliding.at(3).at("counts").at("manifold_points") == 0 &&
          non_colliding.at(3).at("counts").at("events") == 0,
      "kinematic/kinematic admission checkpoint changed");
  const auto& begin = single_contact.at(1);
  expect(
      begin.at("events") == nlohmann::json::parse(
          R"([{"kind":"created","contact":{"fixture_a_id":"contact-static-fixture","child_a":0,"fixture_b_id":"contact-dynamic-fixture","child_b":0,"occurrence":1}},{"kind":"begin","contact":{"fixture_a_id":"contact-static-fixture","child_a":0,"fixture_b_id":"contact-dynamic-fixture","child_b":0,"occurrence":1}},{"kind":"pre_solve","contact":{"fixture_a_id":"contact-static-fixture","child_a":0,"fixture_b_id":"contact-dynamic-fixture","child_b":0,"occurrence":1}},{"kind":"post_solve","contact":{"fixture_a_id":"contact-static-fixture","child_a":0,"fixture_b_id":"contact-dynamic-fixture","child_b":0,"occurrence":1}}])"),
      "contact begin event order or identity changed");
  expect(
      begin.at("contacts").at(0).at("maybe_manifold").at("points").size() == 1,
      "active contact manifold is incomplete");
  expect(
      single_contact.at(3).at("contacts").at(0).at("sensor") &&
          single_contact.at(3).at("contacts").at(0).at("maybe_manifold").is_null(),
      "sensor contact exposed an inactive manifold payload");
  expect(
      single_contact.at(8).at("destructions").at(0).at("kind") == "contact" &&
          single_contact.at(8).at("destructions").at(1).at("kind") == "fixture",
      "fixture teardown order changed");
  expect(
      single_contact.at(9).at("destructions").at(0).at("body_id") ==
              "contact-dynamic" &&
          single_contact.at(9).at("destructions").at(1).at("body_id") ==
              "contact-static",
      "body teardown order changed");
  expect(
      trace.result_record.find("pointer") == std::string::npos &&
          trace.result_record.find("address") == std::string::npos,
      "rigid trace leaked layout identity");
  expect(
      trace.end_record.find("\"reset_verified\":true") != std::string::npos,
      "terminal rigid-world reset proof is missing");
}

void rigid_world_rejects_expanding_ray_clip_during_execution() {
  // Arrange
  auto request = nlohmann::json::parse(read_fixture(
      "protocol/fixtures/accepted/rigid-world-request.jsonl"));
  auto& timelines = request.at("scenario").at("timelines");
  const auto query_timeline = std::find_if(
      timelines.begin(), timelines.end(), [](const auto& timeline) {
        return timeline.at("witness_family") == "world_query_and_ray_cast";
      });
  expect(query_timeline != timelines.end(), "query timeline is missing");
  auto& actions = query_timeline->at("actions");
  const auto ray_action = std::find_if(
      actions.begin(), actions.end(), [](const auto& action) {
        return action.at("action_id") == "query-10";
      });
  expect(ray_action != actions.end(), "clip action is missing");
  ray_action->at("action")["directive_rules"] = nlohmann::json::parse(
      R"([{"target":{"fixture_id":"query-right-fixture","child_index":0},"directive":{"kind":"clip","fraction_bits":1056964608}},{"target":{"fixture_id":"query-center-fixture","child_index":0},"directive":{"kind":"clip","fraction_bits":1061158912}}])");
  RigidWorldAdapter adapter;

  // Act / Assert
  try {
    static_cast<void>(adapter.execute(request.dump() + '\n'));
  } catch (const std::exception& error) {
    expect(
        std::string(error.what()).find("expand current interval") !=
            std::string::npos,
        "expanding clip produced an unstable diagnostic");
    return;
  }
  throw std::runtime_error("expanding ray clip was accepted");
}

void rigid_world_rejects_signed_zero_clips_before_execution() {
  // Arrange and Act / Assert
  for (const auto fraction_bits :
       std::array<std::uint32_t, 2>{0U, 0x80000000U}) {
    auto request = nlohmann::json::parse(read_fixture(
        "protocol/fixtures/accepted/rigid-world-request.jsonl"));
    auto& timeline = query_timeline(request);
    for (auto& body : timeline.at("bodies")) {
      if (body.at("body_id") == "query-left" ||
          body.at("body_id") == "query-center") {
        body["transform"]["position"]["x_bits"] = 0xc0400000U;
      }
    }
    auto& actions = timeline.at("actions");
    const auto ray_action = std::find_if(
        actions.begin(), actions.end(), [](const auto& action) {
          return action.at("action_id") == "query-10";
        });
    expect(ray_action != actions.end(), "clip action is missing");
    ray_action->at("action")["directive_rules"][0]["directive"]
              ["fraction_bits"] = fraction_bits;

    try {
      static_cast<void>(decode_rigid_world_request(request.dump() + '\n'));
    } catch (const std::exception& error) {
      expect(
          std::string(error.what()).find("outside reviewed bounds") !=
              std::string::npos,
          "signed-zero clip produced an unstable diagnostic");
      continue;
    }
    throw std::runtime_error(
        "signed-zero clip reached multiple fraction-zero hit execution");
  }
}

void rigid_world_rejects_invalid_derived_ray_geometry_before_execution() {
  // Arrange and Act / Assert
  const std::array<std::array<std::uint32_t, 4>, 4> endpoints{{
      {0U, 0U, 0x80000000U, 0U},
      {0U, 0U, 1U, 0U},
      {0xff7fffffU, 0U, 0x7f7fffffU, 0U},
      {0U, 0U, 0x7f7fffffU, 0U},
  }};
  for (const auto& endpoint : endpoints) {
    auto request = nlohmann::json::parse(read_fixture(
        "protocol/fixtures/accepted/rigid-world-request.jsonl"));
    auto& actions = query_timeline(request).at("actions");
    const auto ray_action = std::find_if(
        actions.begin(), actions.end(), [](const auto& action) {
          return action.at("action_id") == "query-08";
        });
    expect(ray_action != actions.end(), "ray action is missing");
    ray_action->at("action")["start"] = {
        {"x_bits", endpoint[0]}, {"y_bits", endpoint[1]}};
    ray_action->at("action")["end"] = {
        {"x_bits", endpoint[2]}, {"y_bits", endpoint[3]}};

    try {
      static_cast<void>(decode_rigid_world_request(request.dump() + '\n'));
    } catch (const std::exception& error) {
      expect(
          std::string(error.what()).find("finite non-zero squared direction") !=
              std::string::npos,
          "derived ray geometry produced an unstable diagnostic");
      continue;
    }
    throw std::runtime_error("invalid derived ray geometry reached execution");
  }
}

void rigid_world_rejects_invalid_selector_children_before_execution() {
  // Arrange and Act / Assert
  for (const auto& test_case :
       std::array<std::pair<std::string, std::string>, 2>{
           std::pair{"query-07", "query directive"},
           std::pair{"query-11", "ray directive"}}) {
    const auto& action_id = test_case.first;
    const auto& context = test_case.second;
    auto request = nlohmann::json::parse(read_fixture(
        "protocol/fixtures/accepted/rigid-world-request.jsonl"));
    auto& actions = query_timeline(request).at("actions");
    const auto selected_action = std::find_if(
        actions.begin(), actions.end(), [&](const auto& action) {
          return action.at("action_id") == action_id;
        });
    expect(selected_action != actions.end(), "selector action is missing");
    selected_action->at("action")["directive_rules"][0]["target"]
                   ["child_index"] = 1U;
    RigidWorldAdapter adapter;

    try {
      static_cast<void>(adapter.execute(request.dump() + '\n'));
    } catch (const std::exception& error) {
      expect(
          std::string(error.what()).find(
              context + " references invalid fixture child") !=
              std::string::npos,
          "invalid selector child produced an unstable diagnostic");
      continue;
    }
    throw std::runtime_error(
        "invalid selector child reached rigid-world execution");
  }
}
