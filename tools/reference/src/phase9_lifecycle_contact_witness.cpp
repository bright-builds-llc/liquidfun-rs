// Repository-authored semantic probe for pinned LiquidFun behavior.
// No upstream source or Rust-produced expectation is copied into this file.

#include "build_identity.hpp"
#include "protocol.hpp"

#include <Box2D/Box2D.h>
#include <nlohmann/json.hpp>

#include <algorithm>
#include <chrono>
#include <cstdint>
#include <cstring>
#include <ctime>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <set>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <vector>

namespace {

using Json = nlohmann::json;

constexpr std::string_view kExpectedOracleRevision =
    "7f20402173fd143a3988c921bc384459c6a858f2";
constexpr std::string_view kCmakeTarget = "phase9-lifecycle-contact-witness";
constexpr int32 kEqualExpirationParticleCount = 8;
constexpr int32 kStrictContactFixtureCount = 6;

struct CommandLine {
  std::string output_path;
  std::string provenance_path;
  std::vector<std::string> exact_argv;
};

struct NamedFixture {
  std::string id;
  b2Fixture* fixture = nullptr;
};

struct BodyContactCapture {
  std::vector<std::string> order;
  std::vector<std::uint32_t> weight_bits;
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

std::string particle_id(int32 index) {
  return "particle-" + std::to_string(index);
}

std::string fixture_id(int32 index) {
  return "fixture-" + std::to_string(index);
}

CommandLine parse_command_line(int argc, char** argv) {
  if (argc != 5 || std::string_view(argv[1]) != "--output" ||
      std::string_view(argv[3]) != "--provenance") {
    throw std::runtime_error(
        "usage: phase9-lifecycle-contact-witness --output <path> "
        "--provenance <path>");
  }
  if (std::string_view(argv[2]).empty() || std::string_view(argv[4]).empty() ||
      std::string_view(argv[2]) == std::string_view(argv[4])) {
    throw std::runtime_error("output and provenance paths must be distinct");
  }

  CommandLine command_line;
  command_line.output_path = argv[2];
  command_line.provenance_path = argv[4];
  command_line.exact_argv.reserve(static_cast<std::size_t>(argc));
  for (int index = 0; index < argc; ++index) {
    command_line.exact_argv.emplace_back(argv[index]);
  }
  return command_line;
}

b2ParticleSystem* populate_equal_expiration_system(b2World& world) {
  b2ParticleSystemDef system_definition;
  system_definition.radius = 0.05f;
  system_definition.lifetimeGranularity = 1.0f;
  system_definition.destroyByAge = true;
  b2ParticleSystem* const system =
      world.CreateParticleSystem(&system_definition);
  if (system == nullptr) {
    throw std::runtime_error("failed to create equal-expiration system");
  }

  for (int32 index = 0; index < kEqualExpirationParticleCount; ++index) {
    b2ParticleDef particle_definition;
    particle_definition.position.Set(static_cast<float>(index) * 10.0f, 0.0f);
    particle_definition.lifetime = 2.75f;
    const int32 created_index = system->CreateParticle(particle_definition);
    if (created_index != index) {
      throw std::runtime_error("equal-expiration creation order changed");
    }
  }

  // A positive step calls the pinned SolveLifetimes implementation, which
  // performs the actual std::sort over equal quantized expiration values.
  world.Step(0.25f, 1, 1, 1);
  return system;
}

Json capture_equal_expiration_witness() {
  b2World world(b2Vec2(0.0f, 0.0f));
  b2ParticleSystem* const system = populate_equal_expiration_system(world);
  const int32* const expiration_times = system->GetExpirationTimeBuffer();
  const int32* const expiration_order =
      system->GetIndexByExpirationTimeBuffer();

  const int32 expected_expiration = expiration_times[0];
  Json creation_ids = Json::array();
  Json expiration_ids = Json::array();
  for (int32 index = 0; index < kEqualExpirationParticleCount; ++index) {
    if (expiration_times[index] != expected_expiration) {
      throw std::runtime_error(
          "equal-expiration case produced unequal quantized values");
    }
    creation_ids.push_back(particle_id(index));
    expiration_ids.push_back(particle_id(expiration_order[index]));
  }

  Json oldest_ids = Json::array();
  for (int32 ordinal = 0; ordinal < kEqualExpirationParticleCount; ++ordinal) {
    b2World selection_world(b2Vec2(0.0f, 0.0f));
    b2ParticleSystem* const selection_system =
        populate_equal_expiration_system(selection_world);
    selection_system->DestroyOldestParticle(ordinal, false);

    int32 selected_index = b2_invalidParticleIndex;
    for (int32 index = 0; index < kEqualExpirationParticleCount; ++index) {
      if ((selection_system->GetParticleFlags(index) & b2_zombieParticle) == 0) {
        continue;
      }
      if (selected_index != b2_invalidParticleIndex) {
        throw std::runtime_error("oldest selection marked multiple particles");
      }
      selected_index = index;
    }
    if (selected_index == b2_invalidParticleIndex) {
      throw std::runtime_error("oldest selection did not mark a particle");
    }
    oldest_ids.push_back(particle_id(selected_index));
  }

  return Json{
      {"scenario_id", "equal_quantized_expiration"},
      {"particle_count", kEqualExpirationParticleCount},
      {"quantized_expiration", expected_expiration},
      {"creation_order", creation_ids},
      {"expiration_order", expiration_ids},
      {"oldest_selection_order", oldest_ids},
  };
}

std::string semantic_fixture_id(
    b2Fixture* fixture,
    const std::vector<NamedFixture>& fixtures) {
  const auto found = std::find_if(
      fixtures.begin(), fixtures.end(),
      [fixture](const NamedFixture& named) { return named.fixture == fixture; });
  if (found == fixtures.end()) {
    throw std::runtime_error("body contact referenced an unknown fixture");
  }
  return found->id;
}

BodyContactCapture capture_body_contacts(bool strict_contact_check) {
  b2World world(b2Vec2(0.0f, 0.0f));
  b2CircleShape circle;
  circle.m_radius = 1.0f;

  std::vector<NamedFixture> fixtures;
  fixtures.reserve(kStrictContactFixtureCount);
  for (int32 index = 0; index < kStrictContactFixtureCount; ++index) {
    b2BodyDef body_definition;
    body_definition.position.Set(1.5f, 0.0f);
    b2Body* const body = world.CreateBody(&body_definition);
    b2Fixture* const fixture = body->CreateFixture(&circle, 0.0f);
    fixtures.push_back(NamedFixture{fixture_id(index), fixture});
  }

  b2ParticleSystemDef system_definition;
  system_definition.radius = 1.0f;
  system_definition.strictContactCheck = strict_contact_check;
  b2ParticleSystem* const system =
      world.CreateParticleSystem(&system_definition);
  b2ParticleDef particle_definition;
  particle_definition.position.Set(0.0f, 0.0f);
  if (system == nullptr || system->CreateParticle(particle_definition) != 0) {
    throw std::runtime_error("failed to create strict-contact particle");
  }

  world.Step(1.0f / 60.0f, 1, 1, 1);
  BodyContactCapture capture;
  const b2ParticleBodyContact* const contacts = system->GetBodyContacts();
  const int32 contact_count = system->GetBodyContactCount();
  capture.order.reserve(static_cast<std::size_t>(contact_count));
  capture.weight_bits.reserve(static_cast<std::size_t>(contact_count));
  for (int32 index = 0; index < contact_count; ++index) {
    capture.order.push_back(semantic_fixture_id(contacts[index].fixture, fixtures));
    capture.weight_bits.push_back(float_bits(contacts[index].weight));
  }
  return capture;
}

Json capture_strict_contact_witness() {
  const BodyContactCapture candidates = capture_body_contacts(false);
  const BodyContactCapture kept = capture_body_contacts(true);
  if (candidates.order.size() != kStrictContactFixtureCount ||
      kept.order.empty() || kept.order.size() >= candidates.order.size()) {
    throw std::runtime_error("strict-contact case did not exercise pruning");
  }
  if (candidates.weight_bits.empty() ||
      !std::all_of(
          candidates.weight_bits.begin(), candidates.weight_bits.end(),
          [&](std::uint32_t bits) { return bits == candidates.weight_bits[0]; })) {
    throw std::runtime_error("strict-contact case did not produce an exact weight tie");
  }

  const std::set<std::string> kept_ids(kept.order.begin(), kept.order.end());
  Json outcomes = Json::array();
  for (int32 index = 0; index < kStrictContactFixtureCount; ++index) {
    const std::string id = fixture_id(index);
    outcomes.push_back(Json{
        {"fixture_id", id},
        {"result", kept_ids.find(id) != kept_ids.end() ? "kept" : "removed"},
    });
  }

  return Json{
      {"scenario_id", "strict_contact_pruning"},
      {"fixture_count", kStrictContactFixtureCount},
      {"equal_weight_bits", hexadecimal_bits(candidates.weight_bits[0])},
      {"candidate_order", candidates.order},
      {"strict_order", kept.order},
      {"outcomes", outcomes},
  };
}

std::string utc_timestamp() {
  const std::time_t now = std::chrono::system_clock::to_time_t(
      std::chrono::system_clock::now());
  std::tm utc{};
#if defined(_WIN32)
  if (gmtime_s(&utc, &now) != 0) {
#else
  if (gmtime_r(&now, &utc) == nullptr) {
#endif
    throw std::runtime_error("failed to generate UTC timestamp");
  }
  std::ostringstream output;
  output << std::put_time(&utc, "%Y-%m-%dT%H:%M:%SZ");
  return output.str();
}

void write_json(const std::string& path, const Json& document) {
  std::ofstream output(path, std::ios::binary | std::ios::trunc);
  if (!output) {
    throw std::runtime_error("failed to open output path: " + path);
  }
  output << document.dump(2) << '\n';
  output.flush();
  if (!output) {
    throw std::runtime_error("failed to write output path: " + path);
  }
}

}  // namespace

int main(int argc, char** argv) {
  try {
    const CommandLine command_line = parse_command_line(argc, argv);
    namespace identity = liquidfun::reference::configured_build_identity;
    if (std::string_view(identity::kOracleRevision) != kExpectedOracleRevision) {
      throw std::runtime_error("configured upstream revision is not the pinned oracle");
    }

    const Json witnesses = Json{
        {"schema_version", 1},
        {"oracle_revision", identity::kOracleRevision},
        {"witnesses",
         Json::array({
             capture_equal_expiration_witness(),
             capture_strict_contact_witness(),
         })},
    };
    const std::string witness_bytes = witnesses.dump(2) + '\n';
    const std::string witness_sha256 =
        liquidfun::reference::sha256_hex(witness_bytes);
    write_json(command_line.output_path, witnesses);

    const Json provenance = Json{
        {"schema_version", 1},
        {"oracle_revision", identity::kOracleRevision},
        {"adapter_content_sha256", identity::kAdapterContentSha256},
        {"probe_source_sha256", PHASE9_PROBE_SOURCE_SHA256},
        {"compiler_id", identity::kCompilerId},
        {"compiler_version", identity::kCompilerVersion},
        {"target", identity::kTarget},
        {"cmake_preset", identity::kCmakePreset},
        {"cmake_target", kCmakeTarget},
        {"exact_argv", command_line.exact_argv},
        {"generation_timestamp", utc_timestamp()},
        {"witness_sha256", witness_sha256},
    };
    write_json(command_line.provenance_path, provenance);
    std::cout << "phase9 lifecycle/contact witnesses: " << witness_sha256 << '\n';
    return 0;
  } catch (const std::exception& error) {
    std::cerr << "phase9 lifecycle/contact witness error: " << error.what()
              << '\n';
    return 1;
  }
}
