#include "phase10_group_topology_cases.hpp"

#include <nlohmann/json.hpp>

#include <algorithm>
#include <cmath>
#include <cstdint>
#include <cstring>
#include <iomanip>
#include <sstream>
#include <stdexcept>
#include <string>

#if !defined(_WIN32)
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>
#endif

// This private development probe must observe source-internal cache and solver
// state without modifying the pinned oracle or publishing an FFI surface.
#if defined(__clang__)
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wkeyword-macro"
#endif
#define private public
#include <Box2D/Dynamics/b2World.h>
#include <Box2D/Particle/b2ParticleGroup.h>
#include <Box2D/Particle/b2ParticleSystem.h>
#include <Box2D/Particle/b2VoronoiDiagram.h>
#undef private
#if defined(__clang__)
#pragma clang diagnostic pop
#endif

namespace {

using Json = nlohmann::json;

struct VoronoiCapture {
  int32 count_x = 0;
  int32 count_y = 0;
  int32 node_count = 0;
};

std::uint32_t float_bits(float value) {
  std::uint32_t bits = 0;
  static_assert(sizeof(bits) == sizeof(value));
  std::memcpy(&bits, &value, sizeof(bits));
  return bits;
}

std::string hexadecimal_bits(std::uint32_t bits) {
  std::ostringstream output;
  output << "0x" << std::hex << std::setfill('0') << std::setw(8) << bits;
  return output.str();
}

std::string exact_float(float value) {
  return hexadecimal_bits(float_bits(value));
}

Json exact_vec2(const b2Vec2& value) {
  return Json{{"x", exact_float(value.x)}, {"y", exact_float(value.y)}};
}

Json exact_transform(const b2Transform& value) {
  return Json{
      {"position", exact_vec2(value.p)},
      {"rotation",
       Json{{"sin", exact_float(value.q.s)}, {"cos", exact_float(value.q.c)}}},
  };
}

std::string float_classification(float value) {
  if (std::isnan(value)) {
    return "nan";
  }
  if (std::isinf(value)) {
    return std::signbit(value) ? "negative_infinity" : "positive_infinity";
  }
  return "finite";
}

Json classified_vec2(const b2Vec2& value) {
  return Json{
      {"bits", exact_vec2(value)},
      {"classification",
       Json{{"x", float_classification(value.x)},
            {"y", float_classification(value.y)}}},
  };
}

bool is_finite(const b2Vec2& value) {
  return std::isfinite(value.x) && std::isfinite(value.y);
}

b2TimeStep witness_step() {
  b2TimeStep step{};
  step.dt = 1.0f / 60.0f;
  step.inv_dt = 60.0f;
  step.particleIterations = 1;
  return step;
}

b2ParticleSystem* create_system(b2World& world) {
  b2ParticleSystemDef definition;
  definition.radius = 0.5f;
  definition.gravityScale = 0.0f;
  b2ParticleSystem* const system = world.CreateParticleSystem(&definition);
  if (system == nullptr) {
    throw std::runtime_error("failed to create particle system");
  }
  return system;
}

int32 create_particle(
    b2ParticleSystem& system, const b2Vec2& position, const b2Vec2& velocity,
    std::uint32_t flags) {
  b2ParticleDef definition;
  definition.flags = flags;
  definition.position = position;
  definition.velocity = velocity;
  const int32 index = system.CreateParticle(definition);
  if (index == b2_invalidParticleIndex) {
    throw std::runtime_error("failed to create witness particle");
  }
  return index;
}

Json capture_split_created_metadata() {
  b2World world(b2Vec2_zero);
  b2ParticleSystem* const system = create_system(world);
  int user_data_sentinel = 17;
  const b2Vec2 positions[] = {
      b2Vec2(0.0f, 0.0f),
      b2Vec2(0.5f, 0.0f),
      b2Vec2(4.0f, 0.0f),
      b2Vec2(4.5f, 0.0f),
  };

  b2ParticleGroupDef definition;
  definition.groupFlags =
      b2_solidParticleGroup | b2_particleGroupCanBeEmpty;
  definition.position.Set(1.25f, -2.5f);
  definition.angle = 0.25f;
  definition.strength = 0.375f;
  definition.userData = &user_data_sentinel;
  definition.particleCount = 4;
  definition.positionData = positions;
  b2ParticleGroup* const original = system->CreateParticleGroup(definition);
  if (original == nullptr) {
    throw std::runtime_error("failed to create split witness group");
  }

  system->SplitParticleGroup(original);
  if (system->GetParticleGroupCount() != 2) {
    throw std::runtime_error("split witness did not create exactly two groups");
  }

  b2ParticleGroup* later = system->GetParticleGroupList();
  if (later == original) {
    later = later->GetNext();
  }
  if (later == nullptr || later == original) {
    throw std::runtime_error("split witness did not expose the later component");
  }

  Json members = Json::array();
  for (int32 index = later->m_firstIndex; index < later->m_lastIndex; ++index) {
    members.push_back(Json{
        {"dense_index", index},
        {"position_bits", exact_vec2(system->GetPositionBuffer()[index])},
    });
  }

  const Json cached_statistics = Json{
      {"timestamp", later->m_timestamp},
      {"mass_bits", exact_float(later->m_mass)},
      {"inertia_bits", exact_float(later->m_inertia)},
      {"center_bits", exact_vec2(later->m_center)},
      {"linear_velocity_bits", exact_vec2(later->m_linearVelocity)},
      {"angular_velocity_bits", exact_float(later->m_angularVelocity)},
  };
  const bool cached_statistics_are_zero =
      later->m_timestamp == -1 && float_bits(later->m_mass) == 0 &&
      float_bits(later->m_inertia) == 0 &&
      float_bits(later->m_center.x) == 0 &&
      float_bits(later->m_center.y) == 0 &&
      float_bits(later->m_linearVelocity.x) == 0 &&
      float_bits(later->m_linearVelocity.y) == 0 &&
      float_bits(later->m_angularVelocity) == 0;

  return Json{
      {"id", "split_created_metadata"},
      {"input_bits",
       Json{
           {"positions",
            Json::array(
                {exact_vec2(positions[0]), exact_vec2(positions[1]),
                 exact_vec2(positions[2]), exact_vec2(positions[3])})},
           {"group_flags", definition.groupFlags},
           {"strength", exact_float(definition.strength)},
           {"position", exact_vec2(definition.position)},
           {"angle", exact_float(definition.angle)},
           {"user_data_present", true},
       }},
      {"outcome",
       Json{
           {"group_count", system->GetParticleGroupCount()},
           {"raw_group_flags", later->m_groupFlags},
           {"public_group_flags", later->GetGroupFlags()},
           {"strength_bits", exact_float(later->m_strength)},
           {"transform_bits", exact_transform(later->m_transform)},
           {"user_data_preserved", later->GetUserData() == &user_data_sentinel},
           {"range",
            Json{{"first", later->m_firstIndex}, {"last", later->m_lastIndex}}},
           {"members", members},
           {"cached_statistics", cached_statistics},
           {"cached_statistics_are_exact_zero", cached_statistics_are_zero},
       }},
      {"decision", "preserve_source_behavior"},
  };
}

Json capture_zero_length_pair() {
  b2World world(b2Vec2_zero);
  b2ParticleSystem* const system = create_system(world);
  const b2Vec2 position(0.0f, 0.0f);
  const int32 a =
      create_particle(*system, position, b2Vec2_zero, b2_springParticle);
  const int32 b =
      create_particle(*system, position, b2Vec2_zero, b2_springParticle);

  b2ParticlePair& pair = system->m_pairBuffer.Append();
  pair.indexA = a;
  pair.indexB = b;
  pair.flags = b2_springParticle;
  pair.strength = 0.5f;
  pair.distance = b2Distance(position, position);
  system->SolveSpring(witness_step());

  const b2Vec2 velocity_a = system->GetVelocityBuffer()[a];
  const b2Vec2 velocity_b = system->GetVelocityBuffer()[b];
  const bool finite = is_finite(velocity_a) && is_finite(velocity_b);
  return Json{
      {"id", "zero_length_pair"},
      {"input_bits",
       Json{
           {"position_a", exact_vec2(position)},
           {"position_b", exact_vec2(position)},
           {"velocity_a", exact_vec2(b2Vec2_zero)},
           {"velocity_b", exact_vec2(b2Vec2_zero)},
           {"strength", exact_float(pair.strength)},
           {"step_dt", exact_float(witness_step().dt)},
       }},
      {"outcome",
       Json{
           {"pair_count", system->GetPairCount()},
           {"distance_bits", exact_float(pair.distance)},
           {"velocity_a", classified_vec2(velocity_a)},
           {"velocity_b", classified_vec2(velocity_b)},
           {"all_finite", finite},
       }},
      {"decision", finite ? "preserve_source_behavior" : "typed_error"},
      {"typed_invariant",
       finite ? Json(nullptr) : Json("zero_length_pair_distance")},
  };
}

Json capture_degenerate_triad() {
  b2World world(b2Vec2_zero);
  b2ParticleSystem* const system = create_system(world);
  const b2Vec2 position(0.0f, 0.0f);
  const int32 a =
      create_particle(*system, position, b2Vec2_zero, b2_elasticParticle);
  const int32 b =
      create_particle(*system, position, b2Vec2_zero, b2_elasticParticle);
  const int32 c =
      create_particle(*system, position, b2Vec2_zero, b2_elasticParticle);

  b2ParticleTriad& triad = system->m_triadBuffer.Append();
  triad.indexA = a;
  triad.indexB = b;
  triad.indexC = c;
  triad.flags = b2_elasticParticle;
  triad.strength = 0.75f;
  triad.pa.SetZero();
  triad.pb.SetZero();
  triad.pc.SetZero();
  triad.ka = 0.0f;
  triad.kb = 0.0f;
  triad.kc = 0.0f;
  triad.s = 0.0f;
  system->SolveElastic(witness_step());

  const b2Vec2 velocity_a = system->GetVelocityBuffer()[a];
  const b2Vec2 velocity_b = system->GetVelocityBuffer()[b];
  const b2Vec2 velocity_c = system->GetVelocityBuffer()[c];
  const bool finite =
      is_finite(velocity_a) && is_finite(velocity_b) && is_finite(velocity_c);
  return Json{
      {"id", "degenerate_triad"},
      {"input_bits",
       Json{
           {"position_a", exact_vec2(position)},
           {"position_b", exact_vec2(position)},
           {"position_c", exact_vec2(position)},
           {"pa", exact_vec2(triad.pa)},
           {"pb", exact_vec2(triad.pb)},
           {"pc", exact_vec2(triad.pc)},
           {"ka", exact_float(triad.ka)},
           {"kb", exact_float(triad.kb)},
           {"kc", exact_float(triad.kc)},
           {"s", exact_float(triad.s)},
           {"strength", exact_float(triad.strength)},
       }},
      {"outcome",
       Json{
           {"triad_count", system->GetTriadCount()},
           {"velocity_a", classified_vec2(velocity_a)},
           {"velocity_b", classified_vec2(velocity_b)},
           {"velocity_c", classified_vec2(velocity_c)},
           {"all_finite", finite},
       }},
      {"decision", finite ? "preserve_source_behavior" : "typed_error"},
      {"typed_invariant",
       finite ? Json(nullptr) : Json("degenerate_triad_rest_state")},
  };
}

Json capture_barrier_pair() {
  b2World world(b2Vec2_zero);
  b2ParticleSystem* const system = create_system(world);
  const b2Vec2 endpoint(0.0f, 0.0f);
  const int32 a =
      create_particle(*system, endpoint, b2Vec2_zero, b2_barrierParticle);
  const int32 b =
      create_particle(*system, endpoint, b2Vec2_zero, b2_barrierParticle);
  const int32 c =
      create_particle(*system, b2Vec2(0.25f, 0.0f), b2Vec2_zero, 0);
  system->UpdateContacts(false);

  b2ParticlePair& pair = system->m_pairBuffer.Append();
  pair.indexA = a;
  pair.indexB = b;
  pair.flags = b2_barrierParticle;
  pair.strength = 1.0f;
  pair.distance = 0.0f;
  system->SolveBarrier(witness_step());

  const b2Vec2 velocity_a = system->GetVelocityBuffer()[a];
  const b2Vec2 velocity_b = system->GetVelocityBuffer()[b];
  const b2Vec2 velocity_c = system->GetVelocityBuffer()[c];
  const bool finite =
      is_finite(velocity_a) && is_finite(velocity_b) && is_finite(velocity_c);
  return Json{
      {"id", "barrier_pair"},
      {"input_bits",
       Json{
           {"endpoint_a", exact_vec2(endpoint)},
           {"endpoint_b", exact_vec2(endpoint)},
           {"particle_c", exact_vec2(b2Vec2(0.25f, 0.0f))},
           {"velocity_a", exact_vec2(b2Vec2_zero)},
           {"velocity_b", exact_vec2(b2Vec2_zero)},
           {"velocity_c", exact_vec2(b2Vec2_zero)},
           {"step_dt", exact_float(witness_step().dt)},
       }},
      {"outcome",
       Json{
           {"pair_count", system->GetPairCount()},
           {"velocity_a", classified_vec2(velocity_a)},
           {"velocity_b", classified_vec2(velocity_b)},
           {"velocity_c", classified_vec2(velocity_c)},
           {"pending_force", system->m_hasForce},
           {"all_finite", finite},
       }},
      {"decision", finite ? "preserve_source_behavior" : "typed_error"},
      {"typed_invariant",
       finite ? Json(nullptr) : Json("degenerate_barrier_pair")},
  };
}

VoronoiCapture run_no_necessary_voronoi_case() {
  class CountingCallback : public b2VoronoiDiagram::NodeCallback {
   public:
    void operator()(int32 a, int32 b, int32 c) override {
      B2_NOT_USED(a);
      B2_NOT_USED(b);
      B2_NOT_USED(c);
      ++count;
    }
    int32 count = 0;
  };

  b2StackAllocator allocator;
  b2VoronoiDiagram diagram(&allocator, 2);
  diagram.AddGenerator(b2Vec2(0.0f, 0.0f), 0, false);
  diagram.AddGenerator(b2Vec2(1.0f, 0.0f), 1, false);
  diagram.Generate(0.5f, 1.0f);
  CountingCallback callback;
  diagram.GetNodes(callback);
  return VoronoiCapture{diagram.m_countX, diagram.m_countY, callback.count};
}

Json capture_no_necessary_voronoi() {
  Json outcome;
#if defined(_WIN32)
  outcome = Json{
      {"termination", "not_run_on_windows"},
      {"all_finite", false},
  };
#else
  int descriptors[2];
  if (pipe(descriptors) != 0) {
    throw std::runtime_error("failed to create Voronoi probe pipe");
  }
  const pid_t child = fork();
  if (child < 0) {
    close(descriptors[0]);
    close(descriptors[1]);
    throw std::runtime_error("failed to fork Voronoi probe");
  }
  if (child == 0) {
    close(descriptors[0]);
    const VoronoiCapture capture = run_no_necessary_voronoi_case();
    const ssize_t written =
        write(descriptors[1], &capture, sizeof(VoronoiCapture));
    close(descriptors[1]);
    _exit(written == static_cast<ssize_t>(sizeof(VoronoiCapture)) ? 0 : 2);
  }

  close(descriptors[1]);
  VoronoiCapture capture;
  const ssize_t received =
      read(descriptors[0], &capture, sizeof(VoronoiCapture));
  close(descriptors[0]);
  int status = 0;
  if (waitpid(child, &status, 0) != child) {
    throw std::runtime_error("failed to wait for Voronoi probe");
  }
  if (WIFEXITED(status) && WEXITSTATUS(status) == 0 &&
      received == static_cast<ssize_t>(sizeof(VoronoiCapture))) {
    outcome = Json{
        {"termination", "returned"},
        {"count_x", capture.count_x},
        {"count_y", capture.count_y},
        {"node_count", capture.node_count},
        {"valid_positive_grid", capture.count_x > 0 && capture.count_y > 0},
        {"all_finite", true},
    };
  } else if (WIFSIGNALED(status)) {
    outcome = Json{
        {"termination", "signal"},
        {"signal", WTERMSIG(status)},
        {"all_finite", false},
    };
  } else {
    outcome = Json{
        {"termination", "nonzero_exit"},
        {"exit_code", WIFEXITED(status) ? WEXITSTATUS(status) : -1},
        {"all_finite", false},
    };
  }
#endif

  return Json{
      {"id", "voronoi_no_necessary_generator"},
      {"input_bits",
       Json{
           {"generators",
            Json::array(
                {Json{{"center", exact_vec2(b2Vec2(0.0f, 0.0f))},
                       {"necessary", false}},
                 Json{{"center", exact_vec2(b2Vec2(1.0f, 0.0f))},
                       {"necessary", false}}})},
           {"radius", exact_float(0.5f)},
           {"margin", exact_float(1.0f)},
       }},
      {"outcome", outcome},
      {"decision", "typed_error"},
      {"typed_invariant", "voronoi_requires_necessary_generator"},
  };
}

Json capture_empty_rigid_group() {
  b2World world(b2Vec2_zero);
  b2ParticleSystem* const system = create_system(world);
  b2ParticleGroupDef definition;
  definition.groupFlags =
      b2_rigidParticleGroup | b2_particleGroupCanBeEmpty;
  b2ParticleGroup* const group = system->CreateParticleGroup(definition);
  if (group == nullptr) {
    throw std::runtime_error("failed to create empty rigid group");
  }
  system->SolveRigid(witness_step());

  const float mass = group->GetMass();
  const float inertia = group->GetInertia();
  const b2Vec2 center = group->GetCenter();
  const b2Vec2 linear_velocity = group->GetLinearVelocity();
  const float angular_velocity = group->GetAngularVelocity();
  const bool finite =
      std::isfinite(mass) && std::isfinite(inertia) && is_finite(center) &&
      is_finite(linear_velocity) && std::isfinite(angular_velocity);
  return Json{
      {"id", "rigid_group_empty"},
      {"input_bits",
       Json{
           {"particle_count", 0},
           {"group_flags", definition.groupFlags},
           {"step_dt", exact_float(witness_step().dt)},
       }},
      {"outcome",
       Json{
           {"mass_bits", exact_float(mass)},
           {"inertia_bits", exact_float(inertia)},
           {"center_bits", exact_vec2(center)},
           {"linear_velocity_bits", exact_vec2(linear_velocity)},
           {"angular_velocity_bits", exact_float(angular_velocity)},
           {"transform_bits", exact_transform(group->GetTransform())},
           {"all_finite", finite},
       }},
      {"decision", finite ? "preserve_source_behavior" : "typed_error"},
      {"typed_invariant",
       finite ? Json(nullptr) : Json("empty_rigid_group_state")},
  };
}

Json capture_one_particle_rigid_group() {
  b2World world(b2Vec2_zero);
  b2ParticleSystem* const system = create_system(world);
  const b2Vec2 position(2.0f, -3.0f);
  b2ParticleGroupDef definition;
  definition.groupFlags = b2_rigidParticleGroup;
  definition.linearVelocity.Set(1.5f, -0.25f);
  definition.angularVelocity = 2.0f;
  definition.particleCount = 1;
  definition.positionData = &position;
  b2ParticleGroup* const group = system->CreateParticleGroup(definition);
  if (group == nullptr) {
    throw std::runtime_error("failed to create one-particle rigid group");
  }
  system->SolveRigid(witness_step());

  const float mass = group->GetMass();
  const float inertia = group->GetInertia();
  const b2Vec2 center = group->GetCenter();
  const b2Vec2 linear_velocity = group->GetLinearVelocity();
  const float angular_velocity = group->GetAngularVelocity();
  const b2Vec2 particle_velocity = system->GetVelocityBuffer()[0];
  const bool finite =
      std::isfinite(mass) && std::isfinite(inertia) && is_finite(center) &&
      is_finite(linear_velocity) && std::isfinite(angular_velocity) &&
      is_finite(particle_velocity);
  return Json{
      {"id", "rigid_group_one_particle"},
      {"input_bits",
       Json{
           {"position", exact_vec2(position)},
           {"linear_velocity", exact_vec2(definition.linearVelocity)},
           {"angular_velocity", exact_float(definition.angularVelocity)},
           {"group_flags", definition.groupFlags},
           {"step_dt", exact_float(witness_step().dt)},
       }},
      {"outcome",
       Json{
           {"mass_bits", exact_float(mass)},
           {"inertia_bits", exact_float(inertia)},
           {"center_bits", exact_vec2(center)},
           {"linear_velocity_bits", exact_vec2(linear_velocity)},
           {"angular_velocity_bits", exact_float(angular_velocity)},
           {"particle_velocity", classified_vec2(particle_velocity)},
           {"transform_bits", exact_transform(group->GetTransform())},
           {"all_finite", finite},
       }},
      {"decision", finite ? "preserve_source_behavior" : "typed_error"},
      {"typed_invariant",
       finite ? Json(nullptr) : Json("one_particle_rigid_group_state")},
  };
}

}  // namespace

nlohmann::json capture_phase10_group_topology_cases() {
  return Json::array({
      capture_split_created_metadata(),
      capture_zero_length_pair(),
      capture_degenerate_triad(),
      capture_barrier_pair(),
      capture_no_necessary_voronoi(),
      capture_empty_rigid_group(),
      capture_one_particle_rigid_group(),
  });
}
