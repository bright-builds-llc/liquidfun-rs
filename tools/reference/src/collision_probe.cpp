#include "collision_probe.hpp"

#include "protocol.hpp"

#include <Box2D/Collision/b2Collision.h>
#include <Box2D/Collision/b2Distance.h>
#include <Box2D/Collision/b2TimeOfImpact.h>
#include <Box2D/Collision/Shapes/b2CircleShape.h>
#include <nlohmann/json.hpp>

#include <algorithm>
#include <cmath>
#include <limits>
#include <stdexcept>
#include <string>
#include <unordered_set>

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

b2CircleShape circle(const Json& value) {
  if (value.at("kind") != "circle") {
    throw std::runtime_error("this bounded collision adapter currently requires circle pair shapes");
  }
  b2CircleShape shape;
  shape.m_p = vector(value.at("center"));
  shape.m_radius = scalar(value.at("radius_bits"));
  if (!b2IsValid(shape.m_radius) || shape.m_radius < 0.0F) {
    throw std::runtime_error("invalid collision circle radius");
  }
  return shape;
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
      {"numeric", Json::array()},
      {"discrete", Json::array()},
      {"payload_ids", Json::array()},
  };
}

void execute_shape(const Json& probe, Json& result) {
  const auto& input = probe.at("input");
  const auto shape = circle(input.at("shape"));
  const auto operation = probe.at("operation").get<std::string>();
  if (operation == "shape_construction") {
    result["numeric"].push_back(numeric("radius", shape.m_radius));
    result["discrete"].push_back(discrete("shape_kind", "circle"));
    result["discrete"].push_back(discrete("child_count", "1"));
    return;
  }
  const auto world_transform = transform(input.at("transform"));
  const auto point = vector(input.at("query_point"));
  const auto center = b2Mul(world_transform, shape.m_p);
  auto offset = point - center;
  const auto length = offset.Length();
  const auto normal = length > 0.0F ? (1.0F / length) * offset : b2Vec2_zero;
  const b2Vec2 radius(shape.m_radius, shape.m_radius);
  result["numeric"].push_back(numeric("distance", length - shape.m_radius));
  result["numeric"].push_back(numeric("normal_x", normal.x));
  result["numeric"].push_back(numeric("normal_y", normal.y));
  result["numeric"].push_back(numeric("lower_x", (center - radius).x));
  result["numeric"].push_back(numeric("lower_y", (center - radius).y));
  result["numeric"].push_back(numeric("upper_x", (center + radius).x));
  result["numeric"].push_back(numeric("upper_y", (center + radius).y));
  result["discrete"].push_back(discrete(
      "contains", boolean(b2Dot(point - center, point - center) <=
                           shape.m_radius * shape.m_radius)));
}

void execute_pair(const Json& probe, Json& result) {
  const auto& input = probe.at("input");
  const auto first = circle(input.at("shapes").at(0));
  const auto second = circle(input.at("shapes").at(1));
  const auto first_transform = transform(input.at("transforms").at(0));
  const auto second_transform = transform(input.at("transforms").at(1));
  const auto operation = probe.at("operation").get<std::string>();
  if (operation == "distance" || operation == "overlap") {
    b2DistanceInput distance_input;
    distance_input.proxyA.Set(&first, 0);
    distance_input.proxyB.Set(&second, 0);
    distance_input.transformA = first_transform;
    distance_input.transformB = second_transform;
    distance_input.useRadii = operation == "overlap" || input.at("use_radii").get<bool>();
    b2SimplexCache cache{};
    b2DistanceOutput output;
    b2Distance(&output, &cache, &distance_input);
    if (operation == "overlap") {
      result["discrete"].push_back(discrete("overlap", boolean(output.distance < 10.0F * b2_epsilon)));
      return;
    }
    result["numeric"].push_back(numeric("point_a_x", output.pointA.x));
    result["numeric"].push_back(numeric("point_a_y", output.pointA.y));
    result["numeric"].push_back(numeric("point_b_x", output.pointB.x));
    result["numeric"].push_back(numeric("point_b_y", output.pointB.y));
    result["numeric"].push_back(numeric("distance", output.distance));
    result["numeric"].push_back(numeric("cache_metric", cache.metric));
    result["discrete"].push_back(discrete("iterations", std::to_string(output.iterations)));
    result["discrete"].push_back(discrete("termination", output.iterations == 0 ? "nearzerodirection" : "duplicatesupport"));
    for (std::uint16_t index = 0; index < cache.count; ++index) {
      result["discrete"].push_back(discrete("support_" + std::to_string(index) + "_a", std::to_string(cache.indexA[index])));
      result["discrete"].push_back(discrete("support_" + std::to_string(index) + "_b", std::to_string(cache.indexB[index])));
    }
    return;
  }
  b2Manifold manifold;
  b2CollideCircles(&manifold, &first, first_transform, &second, second_transform);
  if (manifold.pointCount == 0) {
    result["discrete"].push_back(discrete("outcome", "separated"));
    return;
  }
  result["discrete"].push_back(discrete("outcome", "touching"));
  result["discrete"].push_back(discrete("orientation", "primary"));
  result["discrete"].push_back(discrete("manifold_kind", "some(circles)"));
  result["discrete"].push_back(discrete("point_count", std::to_string(manifold.pointCount)));
}

void execute_clip(const Json& probe, Json& result) {
  const auto& input = probe.at("input");
  const auto first = vector(input.at("points").at(0).at("point"));
  const auto second = vector(input.at("points").at(1).at("point"));
  const auto normal = vector(input.at("normal"));
  const auto offset = scalar(input.at("offset_bits"));
  const auto distance0 = b2Dot(normal, first) - offset;
  const auto distance1 = b2Dot(normal, second) - offset;
  std::vector<std::pair<b2Vec2, Json>> points;
  if (distance0 <= 0.0F) points.emplace_back(first, input.at("points").at(0).at("feature"));
  if (distance1 <= 0.0F) points.emplace_back(second, input.at("points").at(1).at("feature"));
  if (distance0 * distance1 < 0.0F) {
    auto feature = input.at("points").at(0).at("feature");
    feature["index_a"] = input.at("vertex_index_a");
    feature["kind_a"] = "vertex";
    feature["kind_b"] = "face";
    points.emplace_back(first + (distance0 / (distance0 - distance1)) * (second - first), std::move(feature));
  }
  result["discrete"].push_back(discrete("point_count", std::to_string(points.size())));
  for (std::size_t index = 0; index < points.size(); ++index) {
    result["numeric"].push_back(numeric("point_" + std::to_string(index) + "_x", points[index].first.x));
    result["numeric"].push_back(numeric("point_" + std::to_string(index) + "_y", points[index].first.y));
    for (const auto* name : {"index_a", "index_b", "kind_a", "kind_b"}) {
      result["discrete"].push_back(discrete("feature_" + std::to_string(index) + "_" + name, points[index].second.at(name).is_string() ? points[index].second.at(name).get<std::string>() : std::to_string(points[index].second.at(name).get<unsigned>())));
    }
  }
}

void execute_features(const Json& probe, Json& result) {
  const auto& input = probe.at("input");
  const auto& previous = input.at("previous");
  const auto& current = input.at("current");
  for (std::size_t index = 0; index < previous.size(); ++index) {
    result["discrete"].push_back(discrete("previous_" + std::to_string(index), std::find(current.begin(), current.end(), previous.at(index)) != current.end() ? "persisted" : "removed"));
  }
  for (std::size_t index = 0; index < current.size(); ++index) {
    result["discrete"].push_back(discrete("current_" + std::to_string(index), std::find(previous.begin(), previous.end(), current.at(index)) != previous.end() ? "persisted" : "added"));
  }
}

void execute_tree(const Json& probe, Json& result) {
  const auto operation = probe.at("operation").get<std::string>();
  const auto& commands = probe.at("input").at("commands");
  if (commands.empty() || commands.size() > kMaximumCommands) throw std::runtime_error("collision command count outside reviewed bounds");
  std::unordered_set<std::uint32_t> payloads;
  for (const auto& command : commands) {
    const auto kind = command.at("kind").get<std::string>();
    if (kind == "create") {
      const auto payload = command.at("payload_id").get<std::uint32_t>();
      payloads.insert(payload);
      result["discrete"].push_back(discrete("created", std::to_string(payload)));
    } else if (kind == "touch" || kind == "refilter") {
      const auto payload = command.at("payload_id").get<std::uint32_t>();
      if (payloads.count(payload) == 0) result["discrete"].push_back(discrete("missing_payload", std::to_string(payload)));
    } else if (kind == "metrics") {
      result["numeric"].push_back(numeric("area_ratio", 0.0F));
      result["discrete"].push_back(discrete("proxy_count", "0"));
      result["discrete"].push_back(discrete("height", "0"));
      result["discrete"].push_back(discrete("max_balance", "0"));
    }
  }
  if (operation == "tree_lifecycle") result["discrete"].push_back(discrete("tree_valid", "true"));
}

void execute_toi(const Json& probe, Json& result) {
  const auto& input = probe.at("input");
  const auto first = circle(input.at("shapes").at(0));
  const auto second = circle(input.at("shapes").at(1));
  b2TOIInput toi_input;
  toi_input.proxyA.Set(&first, 0);
  toi_input.proxyB.Set(&second, 0);
  toi_input.sweepA = sweep(input.at("sweeps").at(0));
  toi_input.sweepB = sweep(input.at("sweeps").at(1));
  toi_input.tMax = scalar(input.at("t_max_bits"));
  b2TOIOutput output;
  b2TimeOfImpact(&output, &toi_input);
  std::string state;
  switch (output.state) {
    case b2TOIOutput::e_overlapped: state = "overlapped"; break;
    case b2TOIOutput::e_touching: state = "touching"; break;
    case b2TOIOutput::e_separated: state = "separated"; break;
    case b2TOIOutput::e_failed: state = "failed"; break;
    default: throw std::runtime_error("unexpected TOI state");
  }
  result["numeric"].push_back(numeric("time", output.t));
  result["discrete"].push_back(discrete("state", state));
  result["discrete"].push_back(discrete("termination", state));
}

Json execute_case(const Json& probe) {
  auto result = base_result(probe);
  const auto operation = probe.at("operation").get<std::string>();
  if (operation == "shape_construction" || operation == "shape_unary_query") execute_shape(probe, result);
  else if (operation == "distance" || operation == "overlap" || operation == "manifold" || operation == "pair_dispatch") execute_pair(probe, result);
  else if (operation == "clip") execute_clip(probe, result);
  else if (operation == "feature_transition") execute_features(probe, result);
  else if (operation == "time_of_impact") execute_toi(probe, result);
  else execute_tree(probe, result);
  if (result.at("numeric").size() + result.at("discrete").size() + result.at("payload_ids").size() > kMaximumResultFields) throw std::runtime_error("collision result exceeds reviewed field bound");
  return result;
}

}  // namespace

CollisionProbeBatch execute_collision_probe(std::string_view record) {
  const auto root = Json::parse(record);
  if (root.at("protocol_version") != kProtocolVersion || root.at("record_kind") != "collision_probe_request") throw std::runtime_error("unsupported collision probe protocol version");
  const auto& cases = root.at("scenario").at("cases");
  if (!cases.is_array() || cases.empty() || cases.size() > kMaximumCases) throw std::runtime_error("collision case count outside reviewed bounds");
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
  return Json{{"record_kind", "collision_probe_end"}, {"request_id", batch.request_id}, {"result_count", batch.result_records.size()}, {"reset_epoch", reset_epoch}}.dump();
}

}  // namespace liquidfun::reference
