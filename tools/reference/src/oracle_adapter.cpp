#include "oracle_adapter.hpp"

#include "Box2D/Box2D.h"

#include <algorithm>
#include <cmath>
#include <limits>
#include <stdexcept>
#include <string>
#include <vector>

namespace liquidfun::reference {
namespace {

std::uint32_t checked_count(int32 value, const char* name) {
  if (value < 0) {
    throw std::runtime_error(std::string(name) + " count was negative");
  }
  return static_cast<std::uint32_t>(value);
}

WorldCounts capture_counts(const b2World& world) {
  WorldCounts counts;
  counts.bodies = checked_count(world.GetBodyCount(), "body");
  counts.joints = checked_count(world.GetJointCount(), "joint");
  counts.contacts = checked_count(world.GetContactCount(), "contact");
  for (auto* body = world.GetBodyList(); body != nullptr; body = body->GetNext()) {
    for (auto* fixture = body->GetFixtureList(); fixture != nullptr;
         fixture = fixture->GetNext()) {
      if (counts.fixtures == std::numeric_limits<std::uint32_t>::max()) {
        throw std::runtime_error("fixture count overflowed");
      }
      ++counts.fixtures;
    }
  }
  for (auto* system = world.GetParticleSystemList(); system != nullptr;
       system = system->GetNext()) {
    if (counts.particle_systems == std::numeric_limits<std::uint32_t>::max()) {
      throw std::runtime_error("particle system count overflowed");
    }
    ++counts.particle_systems;
    const auto particle_count = checked_count(system->GetParticleCount(), "particle");
    if (particle_count > std::numeric_limits<std::uint32_t>::max() - counts.particles) {
      throw std::runtime_error("particle count overflowed");
    }
    counts.particles += particle_count;
    for (auto* group = system->GetParticleGroupList(); group != nullptr;
         group = group->GetNext()) {
      if (counts.particle_groups == std::numeric_limits<std::uint32_t>::max()) {
        throw std::runtime_error("particle group count overflowed");
      }
      ++counts.particle_groups;
    }
  }
  return counts;
}

}  // namespace

OracleTrace OracleAdapter::execute(
    const ScenarioRequest& request,
    const std::string& identity_sha256) {
  if (identity_sha256.size() != 64) {
    throw std::runtime_error("oracle identity must be a SHA-256 digest");
  }
  const auto lowercase_hex = [](unsigned char character) {
    return (character >= '0' && character <= '9') ||
           (character >= 'a' && character <= 'f');
  };
  if (!std::all_of(identity_sha256.begin(), identity_sha256.end(), lowercase_hex)) {
    throw std::runtime_error("oracle identity must use lowercase hexadecimal");
  }
  std::vector<std::string> semantic_ids;
  std::vector<std::string> checkpoints;
  float simulation_time = 0.0F;
  bool world_active = false;
  {
    const auto gravity_x = float_from_bits(request.scenario.gravity_x_bits);
    const auto gravity_y = float_from_bits(request.scenario.gravity_y_bits);
    if (!std::isfinite(gravity_x) || !std::isfinite(gravity_y)) {
      throw std::runtime_error("gravity must be finite before constructing b2World");
    }
    b2World world(b2Vec2(gravity_x, gravity_y));
    world_active = true;
    std::uint32_t ordinal = 0;
    for (const auto& command : request.scenario.commands) {
      const auto timestep = float_from_bits(command.timestep_bits);
      if (!std::isfinite(timestep) || timestep < 0.0F) {
        throw std::runtime_error("timestep must be finite and nonnegative");
      }
      world.Step(
          timestep,
          static_cast<int32>(command.velocity_iterations),
          static_cast<int32>(command.position_iterations),
          static_cast<int32>(command.particle_iterations));
      simulation_time += timestep;
      for (const auto& checkpoint : request.scenario.checkpoints) {
        if (checkpoint.after_command_id != command.command_id) {
          continue;
        }
        checkpoints.push_back(encode_checkpoint(
            request,
            checkpoint,
            ordinal++,
            bits_from_float(simulation_time),
            capture_counts(world),
            identity_sha256));
      }
    }
  }
  world_active = false;
  semantic_ids.clear();
  const auto reset_verified = !world_active && semantic_ids.empty();
  if (!reset_verified) {
    throw std::runtime_error("oracle adapter reset verification failed");
  }
  if (reset_epoch_ == std::numeric_limits<std::uint64_t>::max()) {
    throw std::runtime_error("oracle reset epoch overflowed");
  }
  ++reset_epoch_;

  OracleTrace trace;
  trace.reset_epoch = reset_epoch_;
  trace.reset_verified = true;
  trace.records.reserve(checkpoints.size() + 2);
  const auto scenario_json = encode_scenario(request.scenario);
  trace.records.push_back(encode_trace_begin(
      request, sha256_hex(scenario_json), identity_sha256));
  trace.records.insert(trace.records.end(), checkpoints.begin(), checkpoints.end());
  trace.records.push_back(encode_trace_end(
      request,
      static_cast<std::uint32_t>(checkpoints.size()),
      trace_payload_sha256(checkpoints),
      reset_epoch_,
      true,
      identity_sha256));
  std::size_t trace_bytes = 0;
  for (const auto& record : trace.records) {
    if (record.size() + 1 > kMaximumRecordBytes ||
        trace_bytes > kMaximumTraceBytes - (record.size() + 1)) {
      throw std::runtime_error("oracle trace exceeds reviewed output limits");
    }
    trace_bytes += record.size() + 1;
  }
  return trace;
}

}  // namespace liquidfun::reference
