// Private playground Dam Break Medium/Normal step timer.
// This executable is not the JSONL protocol oracle.

#include <Box2D/Box2D.h>

#include <chrono>
#include <cstdint>
#include <cstdlib>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>

#ifndef DAM_BREAK_BENCH_COMPILER_ID
#define DAM_BREAK_BENCH_COMPILER_ID "unknown"
#endif
#ifndef DAM_BREAK_BENCH_COMPILER_VERSION
#define DAM_BREAK_BENCH_COMPILER_VERSION "unknown"
#endif

namespace {

constexpr float kParticleRadius = 0.06324555f;
constexpr float kParticleSpacing = 0.101193f;
constexpr int32 kParticleColumns = 48;
constexpr int32 kParticleRows = 40;
constexpr int32 kExpectedParticleCount = 48 * 40;
constexpr float kOriginX = -4.7f;
constexpr float kOriginY = 0.4f;
constexpr float kCircleRadius = 0.75f;
constexpr float kCircleX = 2.5f;
constexpr float kCircleY = 5.5f;
constexpr float kGravityY = -10.0f;
constexpr float kDt = 1.0f / 60.0f;
constexpr int32 kVelocityIterations = 8;
constexpr int32 kPositionIterations = 3;
constexpr int32 kParticleIterations = 2;
constexpr int32 kDefaultWarmupSteps = 60;
constexpr int32 kDefaultMeasuredSteps = 600;
constexpr int32 kMaximumParticleCount = 10240;

struct Counts {
  int32 warmup_steps = kDefaultWarmupSteps;
  int32 measured_steps = kDefaultMeasuredSteps;
};

void attach_box(b2Body* body, const b2Vec2 (&vertices)[4]) {
  b2PolygonShape shape;
  shape.Set(vertices, 4);
  b2FixtureDef fixture;
  fixture.shape = &shape;
  fixture.density = 0.0f;
  fixture.friction = 0.2f;
  fixture.restitution = 0.0f;
  body->CreateFixture(&fixture);
}

void build_medium_normal_world(b2World* world) {
  b2BodyDef ground_definition;
  b2Body* const ground = world->CreateBody(&ground_definition);
  const b2Vec2 floor[4] = {
      b2Vec2(-6.0f, -1.0f),
      b2Vec2(6.0f, -1.0f),
      b2Vec2(6.0f, 0.0f),
      b2Vec2(-6.0f, 0.0f),
  };
  const b2Vec2 left_wall[4] = {
      b2Vec2(-6.0f, 0.0f),
      b2Vec2(-5.5f, 0.0f),
      b2Vec2(-5.5f, 8.0f),
      b2Vec2(-6.0f, 8.0f),
  };
  const b2Vec2 right_wall[4] = {
      b2Vec2(5.5f, 0.0f),
      b2Vec2(6.0f, 0.0f),
      b2Vec2(6.0f, 8.0f),
      b2Vec2(5.5f, 8.0f),
  };
  attach_box(ground, floor);
  attach_box(ground, left_wall);
  attach_box(ground, right_wall);

  b2BodyDef circle_definition;
  circle_definition.type = b2_dynamicBody;
  circle_definition.position.Set(kCircleX, kCircleY);
  circle_definition.allowSleep = true;
  b2Body* const circle_body = world->CreateBody(&circle_definition);
  b2CircleShape circle;
  circle.m_p.Set(0.0f, 0.0f);
  circle.m_radius = kCircleRadius;
  b2FixtureDef circle_fixture;
  circle_fixture.shape = &circle;
  circle_fixture.density = 1.0f;
  circle_fixture.friction = 0.2f;
  circle_fixture.restitution = 0.0f;
  circle_body->CreateFixture(&circle_fixture);

  b2ParticleSystemDef system_definition;
  system_definition.radius = kParticleRadius;
  system_definition.maxCount = kMaximumParticleCount;
  b2ParticleSystem* const system = world->CreateParticleSystem(&system_definition);
  for (int32 row = 0; row < kParticleRows; ++row) {
    for (int32 column = 0; column < kParticleColumns; ++column) {
      b2ParticleDef particle;
      particle.flags = b2_waterParticle;
      particle.color = b2ParticleColor(57, 211, 199, 255);
      particle.position.Set(
          kOriginX + static_cast<float>(column) * kParticleSpacing,
          kOriginY + static_cast<float>(row) * kParticleSpacing);
      const int32 created = system->CreateParticle(particle);
      if (created < 0) {
        throw std::runtime_error("CreateParticle failed");
      }
    }
  }
  if (system->GetParticleCount() != kExpectedParticleCount) {
    throw std::runtime_error("particle count is not the locked Medium recipe");
  }
}

int32 parse_non_negative(std::string_view flag, std::string_view raw) {
  char* end = nullptr;
  const long parsed = std::strtol(raw.data(), &end, 10);
  if (end == raw.data() || *end != '\0' || parsed < 0 ||
      parsed > static_cast<long>(INT32_MAX)) {
    throw std::runtime_error(std::string(flag) + " requires a non-negative integer");
  }
  return static_cast<int32>(parsed);
}

Counts parse_counts(int argc, char** argv) {
  Counts counts;
  int index = 1;
  while (index < argc) {
    const std::string_view flag = argv[index];
    if (flag != "--warmup" && flag != "--steps") {
      throw std::runtime_error(
          "unknown argument; expected `--warmup <n>` and/or `--steps <n>`");
    }
    if (index + 1 >= argc) {
      throw std::runtime_error(std::string(flag) + " requires a non-negative integer");
    }
    const int32 value = parse_non_negative(flag, argv[index + 1]);
    if (flag == "--warmup") {
      counts.warmup_steps = value;
    } else {
      counts.measured_steps = value;
    }
    index += 2;
  }
  if (counts.measured_steps <= 0) {
    throw std::runtime_error("--steps must be greater than 0");
  }
  return counts;
}

std::string json_escape(std::string_view value) {
  std::string escaped;
  escaped.reserve(value.size());
  for (const char character : value) {
    if (character == '\\' || character == '"') {
      escaped.push_back('\\');
    }
    escaped.push_back(character);
  }
  return escaped;
}

}  // namespace

int main(int argc, char** argv) {
  try {
    const Counts counts = parse_counts(argc, argv);
    b2World world(b2Vec2(0.0f, kGravityY));
    build_medium_normal_world(&world);

    for (int32 step = 0; step < counts.warmup_steps; ++step) {
      world.Step(kDt, kVelocityIterations, kPositionIterations, kParticleIterations);
    }

    const auto started = std::chrono::steady_clock::now();
    for (int32 step = 0; step < counts.measured_steps; ++step) {
      world.Step(kDt, kVelocityIterations, kPositionIterations, kParticleIterations);
    }
    const std::chrono::duration<double, std::milli> elapsed =
        std::chrono::steady_clock::now() - started;
    const double wall_ms = elapsed.count() > 0.0 ? elapsed.count() : 1e-9;
    const double measured = static_cast<double>(counts.measured_steps);
    const double ms_per_step = wall_ms / measured;
    const double steps_per_s = measured / (wall_ms / 1000.0);
    const double realtime_factor = (measured / 60.0) / (wall_ms / 1000.0);
    std::ostringstream compiler;
    compiler << DAM_BREAK_BENCH_COMPILER_ID << ' '
             << DAM_BREAK_BENCH_COMPILER_VERSION;

    std::cout.setf(std::ios::fixed);
    std::cout.precision(6);
    std::cout << "{\"engine\":\"pinned_cpp\",\"particles\":"
              << kExpectedParticleCount << ",\"warmup_steps\":"
              << counts.warmup_steps << ",\"measured_steps\":"
              << counts.measured_steps << ",\"wall_ms\":" << wall_ms
              << ",\"ms_per_step\":" << ms_per_step
              << ",\"steps_per_s\":" << steps_per_s
              << ",\"realtime_factor\":" << realtime_factor
              << ",\"compiler\":\"" << json_escape(compiler.str()) << "\"}\n";
    return 0;
  } catch (const std::exception& error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
}
