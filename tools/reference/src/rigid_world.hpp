#pragma once

#include <cstddef>
#include <cstdint>
#include <optional>
#include <string>
#include <string_view>
#include <variant>
#include <vector>

class b2Joint;

namespace liquidfun::reference {

inline constexpr std::size_t kRigidWorldMaximumActions = 128;
inline constexpr std::size_t kRigidWorldMaximumDirectives = 128;
inline constexpr std::size_t kRigidWorldMaximumJoints = 64;
inline constexpr std::uint32_t kRigidWorldMaximumIterations = 1024;
inline constexpr std::uint32_t kRigidWorldMaximumContinuousWork = 1000000;
inline constexpr std::uint32_t kRigidWorldTimestepBits = 0x3c888889U;
inline constexpr std::uint32_t kRigidWorldVelocityIterations = 8;
inline constexpr std::uint32_t kRigidWorldPositionIterations = 3;

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
enum class RigidWitnessFamily {
  non_colliding,
  single_contact,
  body_control,
  island_warm_start,
  sleeping_waking,
  continuous_collision,
  continuous_budget,
  query_ray,
  origin_shift,
};
enum class RigidWakePolicy { wake, preserve_sleep };
enum class RigidQueryDirective { continue_query, terminate };
enum class RigidRayDirectiveKind { ignore, terminate, continue_ray, clip };

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
struct SetLinearVelocity { std::string body_id; RigidVec2Bits velocity; };
struct SetAngularVelocity { std::string body_id; std::uint32_t velocity = 0; };
struct ApplyForce {
  std::string body_id;
  RigidVec2Bits force;
  RigidVec2Bits point;
  RigidWakePolicy wake_policy = RigidWakePolicy::wake;
};
struct ApplyTorque {
  std::string body_id;
  std::uint32_t torque = 0;
  RigidWakePolicy wake_policy = RigidWakePolicy::wake;
};
struct ApplyLinearImpulse {
  std::string body_id;
  RigidVec2Bits impulse;
  RigidVec2Bits point;
  RigidWakePolicy wake_policy = RigidWakePolicy::wake;
};
struct ApplyAngularImpulse {
  std::string body_id;
  std::uint32_t impulse = 0;
  RigidWakePolicy wake_policy = RigidWakePolicy::wake;
};
struct SetBodyDamping {
  std::string body_id;
  std::uint32_t linear = 0;
  std::uint32_t angular = 0;
};
struct SetGravityScale { std::string body_id; std::uint32_t scale = 0; };
struct SetFixedRotation { std::string body_id; bool fixed = false; };
struct SetSleepingAllowed { std::string body_id; bool allowed = true; };
struct SetAwake { std::string body_id; bool awake = true; };
struct SetBullet { std::string body_id; bool bullet = false; };
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
struct SetWorldGravity { RigidVec2Bits gravity; };
struct SetAutomaticForceClearing { bool enabled = true; };
struct SetWarmStarting { bool enabled = true; };
struct SetContinuousPhysics { bool enabled = true; };
struct SetSubStepping { bool enabled = false; };
struct ClearForces {};
struct ConfiguredStep {
  std::uint32_t timestep = 0;
  std::uint32_t velocity_iterations = 0;
  std::uint32_t position_iterations = 0;
  std::uint32_t continuous_work_budget = 0;
};
struct RigidAabbBits { RigidVec2Bits lower; RigidVec2Bits upper; };
struct RigidFixtureChildSelector {
  std::string fixture_id;
  std::uint32_t child_index = 0;
};
struct RigidQueryRule {
  RigidFixtureChildSelector target;
  RigidQueryDirective directive = RigidQueryDirective::continue_query;
};
struct QueryAabb {
  RigidAabbBits aabb;
  std::vector<RigidQueryRule> rules;
};
struct RigidRayDirectiveValue {
  RigidRayDirectiveKind kind = RigidRayDirectiveKind::continue_ray;
  std::uint32_t fraction = 0;
};
struct RigidRayRule {
  RigidFixtureChildSelector target;
  RigidRayDirectiveValue directive;
};
struct RayCast {
  RigidVec2Bits start;
  RigidVec2Bits end;
  std::vector<RigidRayRule> rules;
};
struct ShiftOrigin { RigidVec2Bits shift; };
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
    SetLinearVelocity,
    SetAngularVelocity,
    ApplyForce,
    ApplyTorque,
    ApplyLinearImpulse,
    ApplyAngularImpulse,
    SetBodyDamping,
    SetGravityScale,
    SetFixedRotation,
    SetSleepingAllowed,
    SetAwake,
    SetBullet,
    SetFixtureSensor,
    SetFixtureMaterial,
    SetFixtureFilter,
    SetFixtureDensity,
    ResetMassData,
    SetCustomMassData,
    RigidStep,
    SetWorldGravity,
    SetAutomaticForceClearing,
    SetWarmStarting,
    SetContinuousPhysics,
    SetSubStepping,
    ClearForces,
    ConfiguredStep,
    QueryAabb,
    RayCast,
    ShiftOrigin,
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
  std::vector<std::string> phase8_timelines;
  std::vector<std::string> phase9_timelines;
};

struct RigidWorldTrace {
  std::string result_record;
  std::string end_record;
  std::uint64_t reset_epoch = 0;
  bool reset_verified = false;
};

RigidWorldRequest decode_rigid_world_request(std::string_view record);

RigidVec2Bits semantic_phase8_reaction_force_bits(
    const b2Joint& joint,
    float inverse_timestep,
    bool solver_initialized);
bool phase8_reaction_guard_self_test();

class RigidWorldAdapter {
 public:
  RigidWorldTrace execute(std::string_view record);

 private:
  std::uint64_t reset_epoch_ = 0;
};

}  // namespace liquidfun::reference
