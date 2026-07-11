#include "collision_probe.hpp"

#include "protocol.hpp"

#include <Box2D/Collision/Shapes/b2ChainShape.h>
#include <Box2D/Collision/Shapes/b2CircleShape.h>
#include <Box2D/Collision/Shapes/b2EdgeShape.h>
#include <Box2D/Collision/Shapes/b2PolygonShape.h>
#include <Box2D/Collision/b2BroadPhase.h>
#include <Box2D/Collision/b2Collision.h>
#include <Box2D/Collision/b2Distance.h>
#include <Box2D/Collision/b2DynamicTree.h>
#include <Box2D/Collision/b2TimeOfImpact.h>
#include <nlohmann/json.hpp>

#include <algorithm>
#include <cmath>
#include <cstdint>
#include <map>
#include <memory>
#include <optional>
#include <stdexcept>
#include <string>
#include <unordered_set>
#include <utility>
#include <vector>

namespace liquidfun::reference {
namespace {

using Json = nlohmann::json;
constexpr std::size_t kMaximumCases = 256;
constexpr std::size_t kMaximumCommands = 128;
constexpr std::size_t kMaximumResultFields = 128;

float scalar(const Json& value) {
  return float_from_bits(value.get<std::uint32_t>());
}

b2Vec2 vector(const Json& value) {
  return {scalar(value.at("x_bits")), scalar(value.at("y_bits"))};
}

bool finite(const b2Vec2& value) { return b2IsValid(value.x) && b2IsValid(value.y); }

bool same(const b2Vec2& first, const b2Vec2& second) {
  return first.x == second.x && first.y == second.y;
}

b2Transform transform(const Json& value) {
  return {vector(value.at("position")), b2Rot(scalar(value.at("angle_bits")))};
}

b2Sweep sweep(const Json& value) {
  b2Sweep result;
  result.localCenter = vector(value.at("local_center"));
  result.c0 = vector(value.at("initial_center"));
  result.c = vector(value.at("center"));
  result.a0 = scalar(value.at("initial_angle_bits"));
  result.a = scalar(value.at("angle_bits"));
  result.alpha0 = scalar(value.at("initial_fraction_bits"));
  return result;
}

b2AABB command_aabb(const Json& value) {
  b2AABB result;
  result.lowerBound = vector(value.at("lower"));
  result.upperBound = vector(value.at("upper"));
  return result;
}

Json numeric(std::string field, float value) {
  return {{"field", std::move(field)}, {"bits", bits_from_float(value)}};
}

Json discrete(std::string field, std::string value) {
  return {{"field", std::move(field)}, {"value", std::move(value)}};
}

std::string boolean(bool value) { return value ? "true" : "false"; }

Json base_result(const Json& probe) {
  return {
      {"case_id", probe.at("case_id")},
      {"operation", probe.at("operation")},
      {"policy_path", probe.at("policy_path")},
      {"horizon", probe.at("horizon")},
      {"collection_policy", probe.at("collection_policy")},
      {"outcome",
       {{"kind", "accepted"},
        {"numeric", Json::array()},
        {"discrete", Json::array()},
        {"payload_ids", Json::array()}}},
  };
}

Json& numeric_values(Json& result) { return result.at("outcome").at("numeric"); }
Json& discrete_values(Json& result) { return result.at("outcome").at("discrete"); }
Json& payload_ids(Json& result) { return result.at("outcome").at("payload_ids"); }

void label(Json& result, std::string field, std::string value) {
  discrete_values(result).push_back(discrete(std::move(field), std::move(value)));
}

void number(Json& result, std::string field, float value) {
  numeric_values(result).push_back(numeric(std::move(field), value));
}

#include "collision_probe_shapes.hpp"

void execute_shape(const Json& probe, Json& result) {
  const auto& input = probe.at("input");
  const auto shape = build_shape(input.at("shape"));
  const auto operation = probe.at("operation").get<std::string>();
  if (operation == "shape_construction") {
    number(result, "radius", shape->shape()->m_radius);
    label(result, "shape_kind", shape->kind);
    label(result, "child_count", std::to_string(shape->shape()->GetChildCount()));
    return;
  }
  const auto world_transform = transform(input.at("transform"));
  const auto point = vector(input.at("query_point"));
  const auto child = input.at("child_index").get<int32>();
  float distance = 0.0F;
  b2Vec2 normal;
  b2AABB bounds;
  shape->shape()->ComputeDistance(world_transform, point, &distance, &normal, child);
  shape->shape()->ComputeAABB(&bounds, world_transform, child);
  label(result, "contains", boolean(shape->shape()->TestPoint(world_transform, point)));
  number(result, "distance", distance);
  number(result, "normal_x", normal.x);
  number(result, "normal_y", normal.y);
  number(result, "lower_x", bounds.lowerBound.x);
  number(result, "lower_y", bounds.lowerBound.y);
  number(result, "upper_x", bounds.upperBound.x);
  number(result, "upper_y", bounds.upperBound.y);
}

struct PairShapes {
  const b2Shape* first;
  const b2Shape* second;
  b2Transform first_transform;
  b2Transform second_transform;
  std::string first_kind;
  std::string second_kind;
  std::string orientation = "primary";
  b2EdgeShape first_edge;
  b2EdgeShape second_edge;
};

PairShapes canonical_pair(const ShapeValue& first, std::uint32_t child_a, b2Transform transform_a,
                          const ShapeValue& second, std::uint32_t child_b, b2Transform transform_b) {
  PairShapes pair{};
  pair.first = first.shape();
  pair.second = second.shape();
  pair.first_transform = transform_a;
  pair.second_transform = transform_b;
  pair.first_kind = first.kind;
  pair.second_kind = second.kind;
  if (pair.first_kind == "chain") {
    first.chain.GetChildEdge(&pair.first_edge, static_cast<int32>(child_a));
    pair.first = &pair.first_edge;
    pair.first_kind = "edge";
  }
  if (pair.second_kind == "chain") {
    second.chain.GetChildEdge(&pair.second_edge, static_cast<int32>(child_b));
    pair.second = &pair.second_edge;
    pair.second_kind = "edge";
  }
  const bool reverse = (pair.first_kind == "circle" && pair.second_kind != "circle") ||
                       (pair.first_kind == "polygon" && pair.second_kind == "edge");
  if (reverse) {
    std::swap(pair.first, pair.second);
    std::swap(pair.first_transform, pair.second_transform);
    std::swap(pair.first_kind, pair.second_kind);
    pair.orientation = "reversed";
  }
  return pair;
}

void execute_pair(const Json& probe, Json& result) {
  const auto& input = probe.at("input");
  const auto first = build_shape(input.at("shapes").at(0));
  const auto second = build_shape(input.at("shapes").at(1));
  const auto child_a = input.at("child_indices").at(0).get<std::uint32_t>();
  const auto child_b = input.at("child_indices").at(1).get<std::uint32_t>();
  const auto transform_a = transform(input.at("transforms").at(0));
  const auto transform_b = transform(input.at("transforms").at(1));
  const auto operation = probe.at("operation").get<std::string>();
  if (operation == "distance" || operation == "overlap") {
    b2DistanceInput distance_input;
    distance_input.proxyA.Set(first->shape(), static_cast<int32>(child_a));
    distance_input.proxyB.Set(second->shape(), static_cast<int32>(child_b));
    distance_input.transformA = transform_a;
    distance_input.transformB = transform_b;
    distance_input.useRadii = operation == "overlap" || input.at("use_radii").get<bool>();
    b2SimplexCache cache{};
    if (operation == "distance" && !input.at("maybe_cache").is_null()) {
      const auto decision = prepare_cache(input.at("maybe_cache"), *first, child_a, transform_a,
                                          *second, child_b, transform_b);
      label(result, "cache_outcome", decision.outcome);
      if (!decision.reason.empty()) label(result, "cache_reason", decision.reason);
      if (decision.outcome == "rejected") return;
      if (decision.outcome == "used") cache = decision.cache;
    } else if (operation == "distance") {
      label(result, "cache_outcome", "cold");
    }
    b2DistanceOutput output;
    b2Distance(&output, &cache, &distance_input);
    if (operation == "overlap") {
      label(result, "overlap", boolean(output.distance < 10.0F * b2_epsilon));
    } else {
      append_distance(result, output, cache);
    }
    return;
  }
  auto pair = canonical_pair(*first, child_a, transform_a, *second, child_b, transform_b);
  b2Manifold manifold;
  if (pair.first_kind == "circle" && pair.second_kind == "circle") {
    b2CollideCircles(&manifold, static_cast<const b2CircleShape*>(pair.first), pair.first_transform,
                     static_cast<const b2CircleShape*>(pair.second), pair.second_transform);
  } else if (pair.first_kind == "polygon" && pair.second_kind == "circle") {
    b2CollidePolygonAndCircle(&manifold, static_cast<const b2PolygonShape*>(pair.first),
                              pair.first_transform, static_cast<const b2CircleShape*>(pair.second),
                              pair.second_transform);
  } else if (pair.first_kind == "polygon" && pair.second_kind == "polygon") {
    b2CollidePolygons(&manifold, static_cast<const b2PolygonShape*>(pair.first), pair.first_transform,
                      static_cast<const b2PolygonShape*>(pair.second), pair.second_transform);
  } else if (pair.first_kind == "edge" && pair.second_kind == "circle") {
    b2CollideEdgeAndCircle(&manifold, static_cast<const b2EdgeShape*>(pair.first), pair.first_transform,
                           static_cast<const b2CircleShape*>(pair.second), pair.second_transform);
  } else if (pair.first_kind == "edge" && pair.second_kind == "polygon") {
    b2CollideEdgeAndPolygon(&manifold, static_cast<const b2EdgeShape*>(pair.first), pair.first_transform,
                            static_cast<const b2PolygonShape*>(pair.second), pair.second_transform);
  } else {
    label(result, "outcome", "unsupported");
    return;
  }
  if (manifold.pointCount == 0) {
    label(result, "outcome", "separated");
    return;
  }
  label(result, "outcome", "touching");
  label(result, "orientation", pair.orientation);
  const auto kind = manifold.type == b2Manifold::e_circles
                        ? "some(circles)"
                        : manifold.type == b2Manifold::e_faceA ? "some(facea)" : "some(faceb)";
  label(result, "manifold_kind", kind);
  label(result, "point_count", std::to_string(manifold.pointCount));
}

void execute_clip(const Json& probe, Json& result) {
  const auto& input = probe.at("input");
  const auto first = vector(input.at("points").at(0).at("point"));
  const auto second = vector(input.at("points").at(1).at("point"));
  const auto normal = vector(input.at("normal"));
  const auto distance0 = b2Dot(normal, first) - scalar(input.at("offset_bits"));
  const auto distance1 = b2Dot(normal, second) - scalar(input.at("offset_bits"));
  std::vector<std::pair<b2Vec2, Json>> points;
  if (distance0 <= 0.0F) points.emplace_back(first, input.at("points").at(0).at("feature"));
  if (distance1 <= 0.0F) points.emplace_back(second, input.at("points").at(1).at("feature"));
  if (distance0 * distance1 < 0.0F) {
    auto feature = input.at("points").at(0).at("feature");
    feature["index_a"] = input.at("vertex_index_a");
    feature["kind_a"] = "vertex";
    feature["kind_b"] = "face";
    points.emplace_back(first + (distance0 / (distance0 - distance1)) * (second - first), feature);
  }
  label(result, "point_count", std::to_string(points.size()));
  for (std::size_t index = 0; index < points.size(); ++index) {
    number(result, "point_" + std::to_string(index) + "_x", points[index].first.x);
    number(result, "point_" + std::to_string(index) + "_y", points[index].first.y);
    for (const auto* name : {"index_a", "index_b", "kind_a", "kind_b"}) {
      const auto& value = points[index].second.at(name);
      label(result, "feature_" + std::to_string(index) + "_" + name,
            value.is_string() ? value.get<std::string>() : std::to_string(value.get<unsigned>()));
    }
  }
}

void execute_features(const Json& probe, Json& result) {
  const auto& previous = probe.at("input").at("previous");
  const auto& current = probe.at("input").at("current");
  for (std::size_t index = 0; index < previous.size(); ++index) {
    label(result, "previous_" + std::to_string(index),
          std::find(current.begin(), current.end(), previous[index]) != current.end() ? "persisted" : "removed");
  }
  for (std::size_t index = 0; index < current.size(); ++index) {
    label(result, "current_" + std::to_string(index),
          std::find(previous.begin(), previous.end(), current[index]) != previous.end() ? "persisted" : "added");
  }
}

#include "collision_probe_spatial.hpp"

void execute_toi(const Json& probe, Json& result) {
  const auto& input = probe.at("input");
  const auto first = build_shape(input.at("shapes").at(0));
  const auto second = build_shape(input.at("shapes").at(1));
  b2TOIInput toi_input;
  toi_input.proxyA.Set(first->shape(), input.at("child_indices").at(0).get<int32>());
  toi_input.proxyB.Set(second->shape(), input.at("child_indices").at(1).get<int32>());
  toi_input.sweepA = sweep(input.at("sweeps").at(0));
  toi_input.sweepB = sweep(input.at("sweeps").at(1));
  toi_input.tMax = scalar(input.at("t_max_bits"));
  b2TOIOutput output;
  b2TimeOfImpact(&output, &toi_input);
  const auto state = output.state == b2TOIOutput::e_overlapped ? "overlapped"
                     : output.state == b2TOIOutput::e_touching ? "touching"
                     : output.state == b2TOIOutput::e_separated ? "separated"
                                                                 : "failed";
  number(result, "time", output.t);
  label(result, "state", state);
  label(result, "termination", state);
}

Json execute_case(const Json& probe) {
  auto result = base_result(probe);
  if (probe.at("expected_outcome").at("kind") == "rejected") {
    const auto rejection = classify_rejection(probe.at("input").at("shape"),
                                               probe.at("input").at("child_index").get<std::uint32_t>());
    if (rejection.has_value()) {
      result["outcome"] = {{"kind", "rejected"}, {"category", rejection->first}, {"field", rejection->second}};
      return result;
    }
  }
  label(result, "witness_family", probe.at("witness_family").get<std::string>());
  const auto operation = probe.at("operation").get<std::string>();
  if (operation == "shape_construction" || operation == "shape_unary_query") execute_shape(probe, result);
  else if (operation == "distance" || operation == "overlap" || operation == "manifold" || operation == "pair_dispatch") execute_pair(probe, result);
  else if (operation == "clip") execute_clip(probe, result);
  else if (operation == "feature_transition") execute_features(probe, result);
  else if (operation == "time_of_impact") execute_toi(probe, result);
  else if (operation.rfind("broad_phase_", 0) == 0) execute_broad_phase(probe, result);
  else execute_tree(probe, result);
  if (numeric_values(result).size() + discrete_values(result).size() + payload_ids(result).size() >
      kMaximumResultFields) {
    throw std::runtime_error("collision result exceeds reviewed field bound");
  }
  return result;
}

}  // namespace

CollisionProbeBatch execute_collision_probe(std::string_view record) {
  const auto root = Json::parse(record);
  if (root.at("protocol_version") != kProtocolVersion || root.at("record_kind") != "collision_probe_request") {
    throw std::runtime_error("unsupported collision probe protocol version");
  }
  const auto& cases = root.at("scenario").at("cases");
  if (!cases.is_array() || cases.empty() || cases.size() > kMaximumCases) {
    throw std::runtime_error("collision case count outside reviewed bounds");
  }
  CollisionProbeBatch batch{root.at("request_id").get<std::string>(), {}};
  std::unordered_set<std::string> ids;
  for (const auto& probe : cases) {
    const auto id = probe.at("case_id").get<std::string>();
    if (!ids.insert(id).second) throw std::runtime_error("duplicate collision case ID");
    batch.result_records.push_back(execute_case(probe).dump());
  }
  return batch;
}

std::string encode_collision_probe_end(const CollisionProbeBatch& batch, std::uint64_t reset_epoch) {
  return Json{{"record_kind", "collision_probe_end"},
              {"request_id", batch.request_id},
              {"result_count", batch.result_records.size()},
              {"reset_epoch", reset_epoch}}
      .dump();
}

}  // namespace liquidfun::reference
