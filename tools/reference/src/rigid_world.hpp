#pragma once

#include <cstdint>
#include <optional>
#include <string>
#include <string_view>
#include <variant>
#include <vector>

namespace liquidfun::reference {

struct RigidVec2Bits {
  std::uint32_t x = 0;
  std::uint32_t y = 0;
};

struct RigidTransformBits {
  RigidVec2Bits position;
  std::uint32_t angle = 0;
};

struct RigidFilterBits {
  std::uint16_t category = 0;
  std::uint16_t mask = 0;
  std::int16_t group = 0;
};

enum class RigidBodyKind { static_body, kinematic_body, dynamic_body };
enum class RigidWitnessFamily { non_colliding, single_contact };

struct RigidCircleShape {
  RigidVec2Bits center;
  std::uint32_t radius = 0;
};

struct RigidPolygonShape {
  std::vector<RigidVec2Bits> vertices;
};

using RigidShape = std::variant<RigidCircleShape, RigidPolygonShape>;

struct RigidBodyDeclaration {
  std::string id;
  RigidBodyKind kind = RigidBodyKind::static_body;
  RigidTransformBits transform;
  bool active = true;
};

struct RigidFixtureDeclaration {
  std::string id;
  std::string owner_body_id;
  RigidShape shape;
  std::uint32_t density = 0;
  std::uint32_t friction = 0;
  std::uint32_t restitution = 0;
  bool sensor = false;
  RigidFilterBits filter;
};

struct CreateBody { std::string body_id; };
struct CreateFixture { std::string fixture_id; };
struct InspectBody { std::string body_id; };
struct InspectFixture { std::string fixture_id; };
struct SetBodyTransform {
  std::string body_id;
  RigidTransformBits transform;
};
struct SetBodyType {
  std::string body_id;
  RigidBodyKind kind = RigidBodyKind::static_body;
};
struct SetBodyActive {
  std::string body_id;
  bool active = false;
};
struct SetFixtureSensor {
  std::string fixture_id;
  bool sensor = false;
};
struct SetFixtureMaterial {
  std::string fixture_id;
  std::uint32_t friction = 0;
  std::uint32_t restitution = 0;
};
struct SetFixtureFilter {
  std::string fixture_id;
  RigidFilterBits filter;
};
struct SetFixtureDensity {
  std::string fixture_id;
  std::uint32_t density = 0;
};
struct ResetMassData { std::string body_id; };
struct SetCustomMassData {
  std::string body_id;
  std::uint32_t mass = 0;
  RigidVec2Bits center;
  std::uint32_t inertia = 0;
};
struct RigidStep {
  std::uint32_t timestep = 0;
  std::uint32_t velocity_iterations = 0;
  std::uint32_t position_iterations = 0;
};
struct DestroyFixture { std::string fixture_id; };
struct DestroyBody { std::string body_id; };

using RigidAction = std::variant<
    CreateBody,
    CreateFixture,
    InspectBody,
    InspectFixture,
    SetBodyTransform,
    SetBodyType,
    SetBodyActive,
    SetFixtureSensor,
    SetFixtureMaterial,
    SetFixtureFilter,
    SetFixtureDensity,
    ResetMassData,
    SetCustomMassData,
    RigidStep,
    DestroyFixture,
    DestroyBody>;

struct RigidActionRecord {
  std::string id;
  std::string phase;
  RigidAction action;
};

struct RigidExpectedCounts {
  std::uint32_t bodies = 0;
  std::uint32_t fixtures = 0;
  std::uint32_t contacts = 0;
  std::uint32_t manifold_points = 0;
  std::uint32_t events = 0;
  std::uint32_t destructions = 0;
};

struct RigidContactIdentity {
  std::string fixture_a_id;
  std::uint32_t child_a = 0;
  std::string fixture_b_id;
  std::uint32_t child_b = 0;
  std::uint32_t occurrence = 0;
};

struct RigidExpectedTransition {
  std::string witness;
  std::optional<RigidContactIdentity> maybe_contact;
};

struct RigidCheckpoint {
  std::string id;
  std::string after_action_id;
  std::string phase;
  RigidExpectedCounts counts;
  std::vector<RigidExpectedTransition> transitions;
};

struct RigidTimeline {
  RigidWitnessFamily family = RigidWitnessFamily::non_colliding;
  std::vector<RigidBodyDeclaration> bodies;
  std::vector<RigidFixtureDeclaration> fixtures;
  std::vector<RigidActionRecord> actions;
  std::vector<RigidCheckpoint> checkpoints;
};

struct RigidWorldRequest {
  std::string request_id;
  std::string scenario_id;
  std::vector<RigidTimeline> timelines;
};

struct RigidWorldTrace {
  std::string result_record;
  std::string end_record;
  std::uint64_t reset_epoch = 0;
  bool reset_verified = false;
};

RigidWorldRequest decode_rigid_world_request(std::string_view record);

class RigidWorldAdapter {
 public:
  RigidWorldTrace execute(std::string_view record);

 private:
  std::uint64_t reset_epoch_ = 0;
};

}  // namespace liquidfun::reference
