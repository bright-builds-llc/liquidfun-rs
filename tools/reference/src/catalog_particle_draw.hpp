#pragma once

#include "catalog_run_decode.hpp"
#include "protocol.hpp"
#include "Box2D/Box2D.h"

#include <algorithm>
#include <stdexcept>
#include <string>
#include <utility>
#include <vector>

namespace liquidfun::reference::catalog_run_detail {

struct CatalogParticle {
  b2ParticleSystem* system;
  const b2ParticleHandle* handle;
};

inline void retire_catalog_particle_zombies(
    std::vector<std::pair<std::string, CatalogParticle>>& particles) {
  // Catalog particles have no finite lifetimes or age-based capacity eviction.
  // Retire explicit/group zombies while handles are live, before SolveZombie frees
  // them. Zero-dt steps do not solve; positive steps compact even paused systems.
  particles.erase(std::remove_if(particles.begin(), particles.end(), [](const auto& item) {
    const auto& particle = item.second;
    const auto index = particle.handle->GetIndex();
    return (particle.system->GetFlagsBuffer()[index] & b2_zombieParticle) != 0U;
  }), particles.end());
}

class CatalogParticleDraw final : public b2Draw {
 public:
  explicit CatalogParticleDraw(
      const std::vector<std::pair<std::string, CatalogParticle>>& particles)
      : particles_(particles) {
    SetFlags(e_shapeBit | e_jointBit | e_aabbBit | e_pairBit |
             e_centerOfMassBit | e_particleBit);
  }

  void DrawPolygon(const b2Vec2*, int32, const b2Color&) override { ++count; }
  void DrawSolidPolygon(const b2Vec2*, int32, const b2Color&) override {
    ++count;
  }
  void DrawCircle(const b2Vec2&, float32, const b2Color&) override { ++count; }
  void DrawSolidCircle(
      const b2Vec2&,
      float32,
      const b2Vec2&,
      const b2Color&) override {
    ++count;
  }
  void DrawParticles(
      const b2Vec2* positions,
      float32 radius,
      const b2ParticleColor* colors,
      int32 particle_count) override {
    count += static_cast<std::uint32_t>(particle_count);
    for (int32 index = 0; index < particle_count; ++index) {
      const auto found = std::find_if(
          particles_.begin(), particles_.end(), [positions, index](const auto& item) {
            return item.second.system->GetPositionBuffer() == positions &&
                   item.second.handle->GetIndex() == index;
          });
      if (found == particles_.end()) {
        throw std::runtime_error("drawn particle has no semantic catalog ID");
      }
      // DrawParticles receives a null buffer when upstream has never allocated colors.
      // Preserve that state and apply the renderer-neutral transparent-color fallback.
      Json color = {57U, 197U, 207U, 220U};
      if (colors != nullptr && colors[index].a != 0U) {
        color = {colors[index].r, colors[index].g, colors[index].b, colors[index].a};
      }
      const Json key = {
          {"owner", {{"kind", "particle"}, {"semantic_id", found->first}}},
          {"layer", "particles"}, {"kind", "circle"}, {"child", 0U}, {"ordinal", 0U}};
      const Json metadata = {
          {"key", key},
          {"stroke", {{"color", color}, {"width_bits", bits_from_float(0.01F)}}},
          {"maybe_fill", {{"color", color}}}};
      primitives.push_back({
          {"ordering", "source_significant"},
          {"primitive", {{"kind", "circle"}, {"value", {
              {"metadata", metadata},
              {"center", {{"x_bits", bits_from_float(positions[index].x)},
                          {"y_bits", bits_from_float(positions[index].y)}}},
              {"radius_bits", bits_from_float(radius)}}}}}});
    }
  }
  void DrawSegment(const b2Vec2&, const b2Vec2&, const b2Color&) override {
    ++count;
  }
  void DrawTransform(const b2Transform&) override { ++count; }

  std::uint32_t count = 0;
  Json primitives = Json::array();

 private:
  const std::vector<std::pair<std::string, CatalogParticle>>& particles_;
};

}  // namespace liquidfun::reference::catalog_run_detail
