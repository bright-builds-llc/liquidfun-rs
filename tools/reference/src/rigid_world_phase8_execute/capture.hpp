  Json body_snapshots() const {
    Json result = Json::array();
    for (const auto& declaration : timeline_.at("bodies")) {
      const auto id = declaration.at("body_id").get<std::string>();
      const auto found = bodies_.find(id);
      if (found == bodies_.end()) continue;
      const auto& value = *found->second;
      result.push_back(
          {{"body_id", id},
           {"body_kind", rigid_body_kind_name(value.GetType())},
           {"transform", encode_rigid_transform(value)},
           {"active", value.IsActive()},
           {"linear_velocity", encode_rigid_vector(value.GetLinearVelocity())},
           {"angular_velocity_bits", bits_from_float(value.GetAngularVelocity())},
           {"mass_bits", bits_from_float(value.GetMass())},
           {"local_center", encode_rigid_vector(value.GetLocalCenter())},
           {"inertia_bits", bits_from_float(value.GetInertia())}});
    }
    return result;
  }

  Json fixture_snapshots() const {
    Json result = Json::array();
    for (const auto& declaration : timeline_.at("fixtures")) {
      const auto id = declaration.at("fixture_id").get<std::string>();
      const auto found = fixtures_.find(id);
      if (found == fixtures_.end()) continue;
      const auto& value = *found->second;
      result.push_back(
          {{"fixture_id", id},
           {"owner_body_id", declaration.at("owner_body_id")},
           {"sensor", value.IsSensor()},
           {"density_bits", bits_from_float(value.GetDensity())},
           {"friction_bits", bits_from_float(value.GetFriction())},
           {"restitution_bits", bits_from_float(value.GetRestitution())},
           {"filter", encode_rigid_filter(value.GetFilterData())}});
    }
    return result;
  }

  static std::string_view feature_kind(std::uint8_t type) {
    return type == b2ContactFeature::e_vertex ? "vertex" : "face";
  }

  static Json manifold_json(const b2Manifold& manifold) {
    Json points = Json::array();
    for (int32 index = 0; index < manifold.pointCount; ++index) {
      const auto& point = manifold.points[index];
      points.push_back(
          {{"point", encode_rigid_vector(point.localPoint)},
           {"feature",
            {{"index_a", point.id.cf.indexA},
             {"index_b", point.id.cf.indexB},
             {"kind_a", feature_kind(point.id.cf.typeA)},
             {"kind_b", feature_kind(point.id.cf.typeB)}}},
           {"normal_impulse_bits", bits_from_float(point.normalImpulse)},
           {"tangent_impulse_bits", bits_from_float(point.tangentImpulse)}});
    }
    const auto kind = manifold.type == b2Manifold::e_circles
                          ? "circles"
                          : manifold.type == b2Manifold::e_faceA ? "face_a"
                                                                 : "face_b";
    return {
        {"manifold_kind", kind},
        {"local_normal", encode_rigid_vector(manifold.localNormal)},
        {"local_point", encode_rigid_vector(manifold.localPoint)},
        {"points", std::move(points)}};
  }

  Json contact_snapshots() {
    Json result = Json::array();
    for (auto* contact = world_.GetContactList(); contact != nullptr;
         contact = contact->GetNext()) {
      const auto sensor = contact->GetFixtureA()->IsSensor() ||
                          contact->GetFixtureB()->IsSensor();
      Json maybe_manifold = nullptr;
      if (!sensor && contact->GetManifold()->pointCount > 0) {
        maybe_manifold = manifold_json(*contact->GetManifold());
      }
      result.push_back(
          {{"identity", contact_identity(contact)},
           {"touching", contact->IsTouching()},
           {"enabled", contact->IsEnabled()},
           {"sensor", sensor},
           {"mixed_friction_bits", bits_from_float(contact->GetFriction())},
           {"mixed_restitution_bits", bits_from_float(contact->GetRestitution())},
           {"maybe_manifold", std::move(maybe_manifold)}});
    }
    return result;
  }

  Json capture(const Json& checkpoint) {
    if (observations_.size() > kMaximumPhase8Observations) {
      throw std::runtime_error("Phase 8 observation count outside reviewed bounds");
    }
    auto bodies = body_snapshots();
    auto fixtures = fixture_snapshots();
    auto contacts = contact_snapshots();
    std::uint32_t manifold_points = 0;
    for (const auto& contact : contacts) {
      if (!contact.at("maybe_manifold").is_null()) {
        manifold_points += static_cast<std::uint32_t>(
            contact.at("maybe_manifold").at("points").size());
      }
    }
    const Json actual_counts{
        {"bodies", static_cast<std::uint32_t>(bodies.size())},
        {"fixtures", static_cast<std::uint32_t>(fixtures.size())},
        {"contacts", static_cast<std::uint32_t>(contacts.size())},
        {"manifold_points", manifold_points},
        {"events", 0U},
        {"destructions", 0U}};
    if (actual_counts != checkpoint.at("counts")) {
      throw std::runtime_error(
          "Phase 8 checkpoint count mismatch at " +
          checkpoint.at("checkpoint_id").get<std::string>() + ": actual=" +
          actual_counts.dump() + ", expected=" + checkpoint.at("counts").dump());
    }
    Json result{
        {"checkpoint_id", checkpoint.at("checkpoint_id")},
        {"phase", checkpoint.at("phase")},
        {"counts", std::move(actual_counts)},
        {"bodies", std::move(bodies)},
        {"fixtures", std::move(fixtures)},
        {"contacts", std::move(contacts)},
        {"events", Json::array()},
        {"destructions", Json::array()}};
    if (!observations_.empty()) result["observations"] = std::move(observations_);
    observations_ = Json::array();
    return result;
  }

  b2Body& body(const Json& raw_id) { return body(raw_id.get<std::string>()); }
  b2Body& body(const std::string& id) {
    const auto found = bodies_.find(id);
    if (found == bodies_.end()) throw std::runtime_error("Phase 8 body is not live");
    return *found->second;
  }
  b2Fixture& fixture(const Json& raw_id) {
    const auto found = fixtures_.find(raw_id.get<std::string>());
    if (found == fixtures_.end()) throw std::runtime_error("Phase 8 fixture is not live");
    return *found->second;
  }
  b2Joint& joint_json(const Json& raw_id) const {
    return joint_by_id(raw_id.get<std::string>());
  }
  b2Joint& joint_by_id(const std::string& id) const {
    const auto found = joints_.find(id);
    if (found == joints_.end()) throw std::runtime_error("Phase 8 joint is not live");
    return *found->second;
  }
  b2Rope& rope_json(const Json& raw_id) const {
    return rope_by_id(raw_id.get<std::string>());
  }
  b2Rope& rope_by_id(const std::string& id) const {
    const auto found = ropes_.find(id);
    if (found == ropes_.end()) throw std::runtime_error("Phase 8 rope is not live");
    return *found->second;
  }
  std::string semantic_joint(const b2Joint* joint_value) const {
    const auto found = std::find_if(joints_.begin(), joints_.end(), [&](const auto& item) {
      return item.second == joint_value;
    });
    if (found == joints_.end()) throw std::runtime_error("Phase 8 joint identity is unmapped");
    return found->first;
  }

  std::string maybe_semantic_joint(const b2Joint* joint_value) const {
    const auto found = std::find_if(joints_.begin(), joints_.end(), [&](const auto& item) {
      return item.second == joint_value;
    });
    return found == joints_.end() ? std::string{} : found->first;
  }

  std::string semantic_fixture(const b2Fixture* fixture_value) const {
    const auto id = maybe_semantic_fixture(fixture_value);
    if (id.empty()) throw std::runtime_error("Phase 8 fixture identity is unmapped");
    return id;
  }

  std::string maybe_semantic_fixture(const b2Fixture* fixture_value) const {
    const auto found = std::find_if(fixtures_.begin(), fixtures_.end(), [&](const auto& item) {
      return item.second == fixture_value;
    });
    return found == fixtures_.end() ? std::string{} : found->first;
  }

  Json contact_identity(const b2Contact* contact) {
    auto fixture_a_id = semantic_fixture(contact->GetFixtureA());
    auto fixture_b_id = semantic_fixture(contact->GetFixtureB());
    auto child_a = static_cast<std::uint32_t>(contact->GetChildIndexA());
    auto child_b = static_cast<std::uint32_t>(contact->GetChildIndexB());
    if (fixture_order(fixture_b_id) < fixture_order(fixture_a_id)) {
      std::swap(fixture_a_id, fixture_b_id);
      std::swap(child_a, child_b);
    }
    return {
        {"fixture_a_id", fixture_a_id},
        {"child_a", child_a},
        {"fixture_b_id", fixture_b_id},
        {"child_b", child_b},
        {"occurrence", 1}};
  }

  std::size_t fixture_order(const std::string& id) const {
    const auto& declarations = timeline_.at("fixtures");
    const auto found = std::find_if(declarations.begin(), declarations.end(), [&](const auto& item) {
      return item.at("fixture_id") == id;
    });
    if (found == declarations.end()) {
      throw std::runtime_error("Phase 8 fixture declaration is unmapped");
    }
    return static_cast<std::size_t>(std::distance(declarations.begin(), found));
  }
