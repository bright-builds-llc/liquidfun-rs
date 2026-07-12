#include "rigid_world.hpp"

#include "protocol.hpp"
#include "rigid_world_decode.hpp"
#include "rigid_world_trace.hpp"

#include <Box2D/Box2D.h>

#include <algorithm>
#include <limits>
#include <map>
#include <memory>
#include <set>
#include <stdexcept>
#include <string>
#include <tuple>
#include <type_traits>
#include <unordered_map>
#include <utility>

namespace liquidfun::reference {
namespace {

using Json = nlohmann::json;

b2Vec2 vector(RigidVec2Bits value) {
  return {float_from_bits(value.x), float_from_bits(value.y)};
}

b2BodyType body_type(RigidBodyKind kind) {
  switch (kind) {
    case RigidBodyKind::static_body: return b2_staticBody;
    case RigidBodyKind::kinematic_body: return b2_kinematicBody;
    case RigidBodyKind::dynamic_body: return b2_dynamicBody;
  }
  throw std::runtime_error("unreachable rigid body kind");
}

class TimelineExecution;

class SemanticContactListener final : public b2ContactListener {
 public:
  explicit SemanticContactListener(TimelineExecution& execution)
      : execution_(execution) {}

  void BeginContact(b2Contact* contact) override;
  void EndContact(b2Contact* contact) override;
  void PreSolve(b2Contact* contact, const b2Manifold*) override;
  void PostSolve(b2Contact* contact, const b2ContactImpulse*) override;

 private:
  TimelineExecution& execution_;
};

class TimelineExecution {
 public:
  TimelineExecution(b2World& world, const RigidTimeline& timeline)
      : world_(world), timeline_(timeline), listener_(*this) {
    for (const auto& body : timeline_.bodies) body_declarations_.emplace(body.id, &body);
    for (std::size_t index = 0; index < timeline_.fixtures.size(); ++index) {
      const auto& fixture = timeline_.fixtures[index];
      fixture_declarations_.emplace(fixture.id, &fixture);
      fixture_order_.emplace(fixture.id, index);
    }
    world_.SetContactListener(&listener_);
  }

  ~TimelineExecution() { world_.SetContactListener(nullptr); }

  Json run() {
    Json checkpoints = Json::array();
    std::size_t next_checkpoint = 0;
    for (const auto& action : timeline_.actions) {
      execute(action.action);
      if (next_checkpoint < timeline_.checkpoints.size() &&
          timeline_.checkpoints[next_checkpoint].after_action_id == action.id) {
        checkpoints.push_back(capture(timeline_.checkpoints[next_checkpoint]));
        ++next_checkpoint;
      }
    }
    if (next_checkpoint != timeline_.checkpoints.size()) {
      throw std::runtime_error("rigid checkpoint execution was incomplete");
    }
    if (!bodies_.empty() || !fixtures_.empty() || world_.GetBodyCount() != 0 ||
        world_.GetContactCount() != 0 || !contact_identities_.empty()) {
      throw std::runtime_error("rigid timeline did not destroy complete world state");
    }
    return {{"witness_family", rigid_family_name(timeline_.family)},
            {"checkpoints", std::move(checkpoints)}};
  }

  void event(b2Contact* contact, std::string_view kind) {
    const auto& identity = identity_for(contact, kind == "begin");
    events_.push_back(
        {{"kind", kind}, {"contact", encode_rigid_contact_identity(identity)}});
  }

  void persist_before_pre_solve(b2Contact* contact) {
    if (step_start_contacts_.count(contact) && !persisted_.count(contact)) {
      event(contact, "persist");
      persisted_.insert(contact);
    }
    event(contact, "pre_solve");
  }

 private:
  using PairKey =
      std::tuple<std::string, std::uint32_t, std::string, std::uint32_t>;

  const RigidContactIdentity& identity_for(
      const b2Contact* contact,
      bool emit_created) {
    const auto found = contact_identities_.find(contact);
    if (found != contact_identities_.end()) return found->second;
    const auto fixture_a = fixture_ids_.find(contact->GetFixtureA());
    const auto fixture_b = fixture_ids_.find(contact->GetFixtureB());
    if (fixture_a == fixture_ids_.end() || fixture_b == fixture_ids_.end()) {
      throw std::runtime_error("contact references an unmapped fixture");
    }
    auto fixture_a_id = fixture_a->second;
    auto fixture_b_id = fixture_b->second;
    auto child_a = checked_child(contact->GetChildIndexA());
    auto child_b = checked_child(contact->GetChildIndexB());
    if (fixture_order_.at(fixture_b_id) < fixture_order_.at(fixture_a_id)) {
      std::swap(fixture_a_id, fixture_b_id);
      std::swap(child_a, child_b);
    }
    const PairKey key{fixture_a_id, child_a, fixture_b_id, child_b};
    auto& occurrence = occurrences_[key];
    if (occurrence == std::numeric_limits<std::uint32_t>::max()) {
      throw std::runtime_error("contact occurrence overflowed");
    }
    ++occurrence;
    auto [inserted, was_inserted] = contact_identities_.emplace(
        contact,
        RigidContactIdentity{
            fixture_a_id,
            child_a,
            fixture_b_id,
            child_b,
            occurrence});
    if (!was_inserted) throw std::runtime_error("contact identity insertion failed");
    if (emit_created) {
      events_.push_back(
          {{"kind", "created"},
           {"contact", encode_rigid_contact_identity(inserted->second)}});
    }
    return inserted->second;
  }

  static std::uint32_t checked_child(int32 child) {
    if (child < 0) throw std::runtime_error("contact child index was negative");
    return static_cast<std::uint32_t>(child);
  }

  std::set<const b2Contact*> contacts() const {
    std::set<const b2Contact*> result;
    for (auto* contact = world_.GetContactList(); contact != nullptr;
         contact = contact->GetNext()) {
      result.insert(contact);
    }
    return result;
  }

  void begin_action() {
    before_action_contacts_ = contacts();
    step_start_contacts_.clear();
    persisted_.clear();
  }

  void end_action(bool was_step) {
    const auto after = contacts();
    for (const auto* contact : after) {
      const auto is_new = !before_action_contacts_.count(contact);
      static_cast<void>(identity_for(contact, is_new));
      if (was_step && step_start_contacts_.count(contact) && contact->IsTouching() &&
          !persisted_.count(contact)) {
        event(const_cast<b2Contact*>(contact), "persist");
        persisted_.insert(contact);
      }
    }
    for (const auto* contact : before_action_contacts_) {
      if (after.count(contact)) continue;
      const auto found = contact_identities_.find(contact);
      if (found != contact_identities_.end()) record_contact_destruction(found);
    }
  }

  void record_missing_contacts() {
    const auto live = contacts();
    for (const auto* contact : before_action_contacts_) {
      if (live.count(contact)) continue;
      const auto found = contact_identities_.find(contact);
      if (found != contact_identities_.end()) record_contact_destruction(found);
    }
  }

  void record_contact_destruction(
      std::unordered_map<const b2Contact*, RigidContactIdentity>::iterator found) {
    events_.push_back(
        {{"kind", "destroyed"},
         {"contact", encode_rigid_contact_identity(found->second)}});
    destructions_.push_back(
        {{"kind", "contact"},
         {"contact", encode_rigid_contact_identity(found->second)}});
    contact_identities_.erase(found);
  }

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
          } else if constexpr (std::is_same_v<T, DestroyFixture>) {
            destroy_fixture(current.fixture_id);
          } else if constexpr (std::is_same_v<T, DestroyBody>) {
            destroy_body(current.body_id);
          }
        },
        action);
    end_action(was_step);
  }

  void create_body(const std::string& id) {
    const auto declaration = body_declarations_.find(id);
    if (declaration == body_declarations_.end()) {
      throw std::runtime_error("body declaration was not found");
    }
    b2BodyDef definition;
    definition.type = body_type(declaration->second->kind);
    definition.position = vector(declaration->second->transform.position);
    definition.angle = float_from_bits(declaration->second->transform.angle);
    definition.active = declaration->second->active;
    definition.allowSleep = false;
    auto* created = world_.CreateBody(&definition);
    if (created == nullptr || !bodies_.emplace(id, created).second) {
      throw std::runtime_error("pinned world failed to create body");
    }
  }

  void create_fixture(const std::string& id) {
    const auto declaration = fixture_declarations_.find(id);
    if (declaration == fixture_declarations_.end()) {
      throw std::runtime_error("fixture declaration was not found");
    }
    b2FixtureDef definition;
    b2CircleShape circle;
    b2PolygonShape polygon;
    if (const auto* circle_shape =
            std::get_if<RigidCircleShape>(&declaration->second->shape)) {
      circle.m_p = vector(circle_shape->center);
      circle.m_radius = float_from_bits(circle_shape->radius);
      definition.shape = &circle;
    } else {
      const auto& polygon_shape =
          std::get<RigidPolygonShape>(declaration->second->shape);
      std::vector<b2Vec2> vertices;
      for (const auto vertex : polygon_shape.vertices) {
        vertices.push_back(vector(vertex));
      }
      polygon.Set(vertices.data(), static_cast<int32>(vertices.size()));
      definition.shape = &polygon;
    }
    definition.density = float_from_bits(declaration->second->density);
    definition.friction = float_from_bits(declaration->second->friction);
    definition.restitution = float_from_bits(declaration->second->restitution);
    definition.isSensor = declaration->second->sensor;
    definition.filter.categoryBits = declaration->second->filter.category;
    definition.filter.maskBits = declaration->second->filter.mask;
    definition.filter.groupIndex = declaration->second->filter.group;
    auto* created = body(declaration->second->owner_body_id).CreateFixture(&definition);
    if (created == nullptr || !fixtures_.emplace(id, created).second ||
        !fixture_ids_.emplace(created, id).second) {
      throw std::runtime_error("pinned body failed to create fixture");
    }
  }

  void destroy_fixture(const std::string& id) {
    auto found = fixtures_.find(id);
    if (found == fixtures_.end()) throw std::runtime_error("fixture is not live");
    auto* target = found->second;
    target->GetBody()->DestroyFixture(target);
    record_missing_contacts();
    fixture_ids_.erase(target);
    fixtures_.erase(found);
    destructions_.push_back({{"kind", "fixture"}, {"fixture_id", id}});
  }

  void destroy_body(const std::string& id) {
    auto found = bodies_.find(id);
    if (found == bodies_.end()) throw std::runtime_error("body is not live");
    auto* target = found->second;
    for (auto fixture = fixtures_.begin(); fixture != fixtures_.end();) {
      if (fixture->second->GetBody() == target) {
        fixture_ids_.erase(fixture->second);
        fixture = fixtures_.erase(fixture);
      } else {
        ++fixture;
      }
    }
    world_.DestroyBody(target);
    record_missing_contacts();
    bodies_.erase(found);
    destructions_.push_back({{"kind", "body"}, {"body_id", id}});
  }

  b2Body& body(const std::string& id) {
    const auto found = bodies_.find(id);
    if (found == bodies_.end()) throw std::runtime_error("body is not live");
    return *found->second;
  }

  b2Fixture& fixture(const std::string& id) {
    const auto found = fixtures_.find(id);
    if (found == fixtures_.end()) throw std::runtime_error("fixture is not live");
    return *found->second;
  }

  Json body_snapshots() const {
    Json result = Json::array();
    for (const auto& declaration : timeline_.bodies) {
      const auto found = bodies_.find(declaration.id);
      if (found == bodies_.end()) continue;
      const auto& value = *found->second;
      result.push_back(
          {{"body_id", declaration.id},
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
    for (const auto& declaration : timeline_.fixtures) {
      const auto found = fixtures_.find(declaration.id);
      if (found == fixtures_.end()) continue;
      const auto& value = *found->second;
      result.push_back(
          {{"fixture_id", declaration.id},
           {"owner_body_id", declaration.owner_body_id},
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

  Json manifold_json(const b2Manifold& manifold) const {
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
    return {{"manifold_kind", kind},
            {"local_normal", encode_rigid_vector(manifold.localNormal)},
            {"local_point", encode_rigid_vector(manifold.localPoint)},
            {"points", std::move(points)}};
  }

  Json contact_snapshots() {
    Json result = Json::array();
    for (auto* contact = world_.GetContactList(); contact != nullptr;
         contact = contact->GetNext()) {
      const auto& identity = identity_for(contact, false);
      const auto sensor = contact->GetFixtureA()->IsSensor() ||
                          contact->GetFixtureB()->IsSensor();
      Json maybe_manifold = nullptr;
      if (!sensor && contact->GetManifold()->pointCount > 0) {
        maybe_manifold = manifold_json(*contact->GetManifold());
      }
      result.push_back(
          {{"identity", encode_rigid_contact_identity(identity)},
           {"touching", contact->IsTouching()},
           {"enabled", contact->IsEnabled()},
           {"sensor", sensor},
           {"mixed_friction_bits", bits_from_float(contact->GetFriction())},
           {"mixed_restitution_bits", bits_from_float(contact->GetRestitution())},
           {"maybe_manifold", std::move(maybe_manifold)}});
    }
    return result;
  }

  Json capture(const RigidCheckpoint& checkpoint) {
    auto bodies = body_snapshots();
    auto fixtures = fixture_snapshots();
    auto contacts = contact_snapshots();
    std::uint32_t point_count = 0;
    for (const auto& contact : contacts) {
      if (!contact.at("maybe_manifold").is_null()) {
        point_count += static_cast<std::uint32_t>(
            contact.at("maybe_manifold").at("points").size());
      }
    }
    RigidExpectedCounts actual{
        static_cast<std::uint32_t>(bodies.size()),
        static_cast<std::uint32_t>(fixtures.size()),
        static_cast<std::uint32_t>(contacts.size()),
        point_count,
        static_cast<std::uint32_t>(events_.size()),
        static_cast<std::uint32_t>(destructions_.size())};
    validate_checkpoint_contact_identity(checkpoint, contacts);
    if (std::tie(
            actual.bodies,
            actual.fixtures,
            actual.contacts,
            actual.manifold_points,
            actual.events,
            actual.destructions) !=
        std::tie(
            checkpoint.counts.bodies,
            checkpoint.counts.fixtures,
            checkpoint.counts.contacts,
            checkpoint.counts.manifold_points,
            checkpoint.counts.events,
            checkpoint.counts.destructions)) {
      throw std::runtime_error(
          "pinned rigid checkpoint count mismatch at " + checkpoint.id +
          ": actual=" + std::to_string(actual.bodies) + "," +
          std::to_string(actual.fixtures) + "," +
          std::to_string(actual.contacts) + "," +
          std::to_string(actual.manifold_points) + "," +
          std::to_string(actual.events) + "," +
          std::to_string(actual.destructions) + " expected=" +
          std::to_string(checkpoint.counts.bodies) + "," +
          std::to_string(checkpoint.counts.fixtures) + "," +
          std::to_string(checkpoint.counts.contacts) + "," +
          std::to_string(checkpoint.counts.manifold_points) + "," +
          std::to_string(checkpoint.counts.events) + "," +
          std::to_string(checkpoint.counts.destructions));
    }
    Json result{
        {"checkpoint_id", checkpoint.id},
        {"phase", checkpoint.phase},
        {"counts", encode_rigid_counts(actual)},
        {"bodies", std::move(bodies)},
        {"fixtures", std::move(fixtures)},
        {"contacts", std::move(contacts)},
        {"events", std::move(events_)},
        {"destructions", std::move(destructions_)}};
    events_ = Json::array();
    destructions_ = Json::array();
    return result;
  }

  void validate_checkpoint_contact_identity(
      const RigidCheckpoint& checkpoint,
      const Json& contacts) const {
    const auto expected = std::find_if(
        checkpoint.transitions.begin(),
        checkpoint.transitions.end(),
        [](const auto& transition) {
          return transition.maybe_contact.has_value();
        });
    if (expected == checkpoint.transitions.end()) return;
    const auto expected_json =
        encode_rigid_contact_identity(*expected->maybe_contact);
    const auto check = [&](const Json& actual) {
      if (actual != expected_json) {
        throw std::runtime_error(
            "pinned contact identity disagrees with declaration at " +
            checkpoint.id);
      }
    };
    for (const auto& contact : contacts) check(contact.at("identity"));
    for (const auto& event : events_) check(event.at("contact"));
    for (const auto& destruction : destructions_) {
      if (destruction.at("kind") == "contact") check(destruction.at("contact"));
    }
  }

  b2World& world_;
  const RigidTimeline& timeline_;
  SemanticContactListener listener_;
  std::unordered_map<std::string, const RigidBodyDeclaration*> body_declarations_;
  std::unordered_map<std::string, const RigidFixtureDeclaration*> fixture_declarations_;
  std::unordered_map<std::string, std::size_t> fixture_order_;
  std::unordered_map<std::string, b2Body*> bodies_;
  std::unordered_map<std::string, b2Fixture*> fixtures_;
  std::unordered_map<const b2Fixture*, std::string> fixture_ids_;
  std::unordered_map<const b2Contact*, RigidContactIdentity> contact_identities_;
  std::map<PairKey, std::uint32_t> occurrences_;
  std::set<const b2Contact*> before_action_contacts_;
  std::set<const b2Contact*> step_start_contacts_;
  std::set<const b2Contact*> persisted_;
  Json events_ = Json::array();
  Json destructions_ = Json::array();

  friend class SemanticContactListener;
};

void SemanticContactListener::BeginContact(b2Contact* contact) {
  execution_.event(contact, "begin");
}

void SemanticContactListener::EndContact(b2Contact* contact) {
  execution_.event(contact, "end");
}

void SemanticContactListener::PreSolve(
    b2Contact* contact,
    const b2Manifold*) {
  execution_.persist_before_pre_solve(contact);
}

void SemanticContactListener::PostSolve(
    b2Contact* contact,
    const b2ContactImpulse*) {
  execution_.event(contact, "post_solve");
}

}  // namespace

RigidWorldRequest decode_rigid_world_request(std::string_view record) {
  return rigid_world_decode::decode(record);
}

RigidWorldTrace RigidWorldAdapter::execute(std::string_view record) {
  const auto request = decode_rigid_world_request(record);
  Json timeline_results = Json::array();
  bool world_active = false;
  {
    b2World world({0.0F, 0.0F});
    world.SetAllowSleeping(false);
    world.SetContinuousPhysics(false);
    world_active = true;
    for (const auto& timeline : request.timelines) {
      TimelineExecution execution(world, timeline);
      timeline_results.push_back(execution.run());
    }
    if (world.GetBodyCount() != 0 || world.GetContactCount() != 0) {
      throw std::runtime_error("rigid request left pinned world state live");
    }
  }
  world_active = false;
  if (world_active) throw std::runtime_error("rigid world reset proof failed");
  if (reset_epoch_ == std::numeric_limits<std::uint64_t>::max()) {
    throw std::runtime_error("rigid world reset epoch overflowed");
  }
  ++reset_epoch_;
  Json result{
      {"protocol_version", 1},
      {"record_kind", "rigid_world_result"},
      {"request_id", request.request_id},
      {"trace_schema_version", 1},
      {"scenario_id", request.scenario_id},
      {"timelines", std::move(timeline_results)}};
  auto result_record = encode_rigid_world_result(result);
  auto end_record = encode_rigid_world_end(
      request.request_id, 1, reset_epoch_);
  if (result_record.size() + end_record.size() + 2 > kMaximumTraceBytes) {
    throw std::runtime_error("rigid world trace exceeds reviewed output limit");
  }
  return {
      std::move(result_record), std::move(end_record), reset_epoch_, true};
}

}  // namespace liquidfun::reference
