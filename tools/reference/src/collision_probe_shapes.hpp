#pragma once

struct ShapeValue {
  std::string kind;
  b2CircleShape circle;
  b2EdgeShape edge;
  b2PolygonShape polygon;
  b2ChainShape chain;

  const b2Shape* shape() const {
    if (kind == "circle") return &circle;
    if (kind == "edge") return &edge;
    if (kind == "polygon") return &polygon;
    if (kind == "chain") return &chain;
    throw std::runtime_error("unknown collision shape kind");
  }
};

std::vector<b2Vec2> vertices(const Json& value) {
  std::vector<b2Vec2> result;
  result.reserve(value.size());
  for (const auto& point : value) result.push_back(vector(point));
  return result;
}
std::unique_ptr<ShapeValue> build_shape(const Json& value) {
  auto result = std::make_unique<ShapeValue>();
  result->kind = value.at("kind").get<std::string>();
  if (result->kind == "circle") {
    result->circle.m_p = vector(value.at("center"));
    result->circle.m_radius = scalar(value.at("radius_bits"));
  } else if (result->kind == "edge") {
    result->edge.Set(vector(value.at("start")), vector(value.at("end")));
    if (!value.at("maybe_previous").is_null()) {
      result->edge.m_vertex0 = vector(value.at("maybe_previous"));
      result->edge.m_hasVertex0 = true;
    }
    if (!value.at("maybe_next").is_null()) {
      result->edge.m_vertex3 = vector(value.at("maybe_next"));
      result->edge.m_hasVertex3 = true;
    }
  } else if (result->kind == "polygon") {
    const auto points = vertices(value.at("vertices"));
    result->polygon.Set(points.data(), static_cast<int32>(points.size()));
  } else if (result->kind == "chain") {
    const auto points = vertices(value.at("vertices"));
    if (value.at("closed").get<bool>()) {
      result->chain.CreateLoop(points.data(), static_cast<int32>(points.size()));
    } else {
      result->chain.CreateChain(points.data(), static_cast<int32>(points.size()));
      if (!value.at("maybe_previous").is_null()) {
        result->chain.SetPrevVertex(vector(value.at("maybe_previous")));
      }
      if (!value.at("maybe_next").is_null()) {
        result->chain.SetNextVertex(vector(value.at("maybe_next")));
      }
    }
  } else {
    throw std::runtime_error("unknown collision shape kind");
  }
  return result;
}

using Rejection = std::pair<std::string, std::string>;

std::optional<Rejection> classify_rejection(const Json& shape, std::uint32_t child) {
  const auto kind = shape.at("kind").get<std::string>();
  if (kind == "circle") {
    const auto center = vector(shape.at("center"));
    const auto radius = scalar(shape.at("radius_bits"));
    if (!finite(center)) return Rejection{"non_finite_value", "circle_center"};
    if (!b2IsValid(radius)) return Rejection{"non_finite_value", "circle_radius"};
    if (radius < 0.0F) return Rejection{"invalid_geometry", "circle_radius"};
    if (child != 0) return Rejection{"invalid_child_index", "child_index"};
    return std::nullopt;
  }
  if (kind == "edge") {
    const auto start = vector(shape.at("start"));
    const auto end = vector(shape.at("end"));
    if (!finite(start)) return Rejection{"non_finite_value", "edge_start"};
    if (!finite(end)) return Rejection{"non_finite_value", "edge_end"};
    if (same(start, end)) return Rejection{"invalid_geometry", "edge_end"};
    if (!shape.at("maybe_previous").is_null()) {
      const auto previous = vector(shape.at("maybe_previous"));
      if (!finite(previous)) return Rejection{"non_finite_value", "edge_previous"};
      if (same(previous, start)) return Rejection{"invalid_geometry", "edge_previous"};
    }
    if (!shape.at("maybe_next").is_null()) {
      const auto next = vector(shape.at("maybe_next"));
      if (!finite(next)) return Rejection{"non_finite_value", "edge_next"};
      if (same(next, end)) return Rejection{"invalid_geometry", "edge_next"};
    }
    if (child != 0) return Rejection{"invalid_child_index", "child_index"};
    return std::nullopt;
  }
  const auto points = vertices(shape.at("vertices"));
  if (std::any_of(points.begin(), points.end(), [](const b2Vec2& point) { return !finite(point); })) {
    return Rejection{"non_finite_value", kind == "polygon" ? "polygon_vertices" : "chain_vertices"};
  }
  if (kind == "polygon") {
    if (points.size() < 3 || points.size() > b2_maxPolygonVertices) {
      return Rejection{"invalid_geometry", "polygon_vertices"};
    }
    std::vector<b2Vec2> welded;
    for (const auto& point : points) {
      const bool unique = std::all_of(welded.begin(), welded.end(), [&](const b2Vec2& existing) {
        return (point - existing).LengthSquared() >= 0.5F * b2_linearSlop;
      });
      if (unique) welded.push_back(point);
    }
    if (welded.size() < 3) return Rejection{"invalid_geometry", "polygon_vertices"};
    b2PolygonShape polygon;
    polygon.Set(points.data(), static_cast<int32>(points.size()));
    if (!polygon.Validate()) return Rejection{"invalid_geometry", "polygon_vertices"};
    if (child != 0) return Rejection{"invalid_child_index", "child_index"};
    return std::nullopt;
  }
  if (kind != "chain") throw std::runtime_error("unknown collision shape kind");
  const bool closed = shape.at("closed").get<bool>();
  if (points.size() < (closed ? 3U : 2U)) return Rejection{"invalid_geometry", "chain_vertices"};
  for (std::size_t index = 1; index < points.size(); ++index) {
    if ((points[index] - points[index - 1]).LengthSquared() <= b2_linearSlop * b2_linearSlop) {
      return Rejection{"invalid_geometry", "chain_vertices"};
    }
  }
  if (closed && (points.back() - points.front()).LengthSquared() <= b2_linearSlop * b2_linearSlop) {
    return Rejection{"invalid_geometry", "chain_vertices"};
  }
  const auto child_count = closed ? points.size() : points.size() - 1;
  if (child >= child_count) return Rejection{"invalid_child_index", "child_index"};
  return std::nullopt;
}

Json proxy_fingerprint(const ShapeValue& value, std::uint32_t child) {
  Json points = Json::array();
  const b2Shape* shape = value.shape();
  if (value.kind == "circle") {
    points.push_back({{"x_bits", bits_from_float(value.circle.m_p.x)}, {"y_bits", bits_from_float(value.circle.m_p.y)}});
  } else if (value.kind == "edge") {
    for (const auto& point : {value.edge.m_vertex1, value.edge.m_vertex2}) {
      points.push_back({{"x_bits", bits_from_float(point.x)}, {"y_bits", bits_from_float(point.y)}});
    }
  } else if (value.kind == "polygon") {
    for (int32 index = 0; index < value.polygon.m_count; ++index) {
      const auto point = value.polygon.m_vertices[index];
      points.push_back({{"x_bits", bits_from_float(point.x)}, {"y_bits", bits_from_float(point.y)}});
    }
  } else {
    b2EdgeShape edge;
    value.chain.GetChildEdge(&edge, static_cast<int32>(child));
    for (const auto& point : {edge.m_vertex1, edge.m_vertex2}) {
      points.push_back({{"x_bits", bits_from_float(point.x)}, {"y_bits", bits_from_float(point.y)}});
    }
  }
  return {{"shape_kind", value.kind},
          {"child_index", child},
          {"radius_bits", bits_from_float(shape->m_radius)},
          {"vertices", std::move(points)}};
}

struct CacheDecision {
  std::string outcome;
  std::string reason;
  b2SimplexCache cache{};
};

CacheDecision prepare_cache(const Json& seed, const ShapeValue& first, std::uint32_t child_a,
                            const b2Transform& transform_a, const ShapeValue& second,
                            std::uint32_t child_b, const b2Transform& transform_b) {
  CacheDecision decision;
  if (seed.at("proxy_a") != proxy_fingerprint(first, child_a)) {
    return {"rejected", "proxy_a_fingerprint_mismatch", {}};
  }
  if (seed.at("proxy_b") != proxy_fingerprint(second, child_b)) {
    return {"rejected", "proxy_b_fingerprint_mismatch", {}};
  }
  const auto& pairs = seed.at("support_pairs");
  if (pairs.empty() || pairs.size() > 3) return {"rejected", "support_count_out_of_range", {}};
  b2DistanceProxy proxy_a;
  b2DistanceProxy proxy_b;
  proxy_a.Set(first.shape(), static_cast<int32>(child_a));
  proxy_b.Set(second.shape(), static_cast<int32>(child_b));
  std::unordered_set<std::uint64_t> seen;
  decision.cache.count = static_cast<std::uint16_t>(pairs.size());
  for (std::size_t index = 0; index < pairs.size(); ++index) {
    const auto index_a = pairs[index].at("index_a").get<std::uint32_t>();
    const auto index_b = pairs[index].at("index_b").get<std::uint32_t>();
    if (index_a >= static_cast<std::uint32_t>(proxy_a.GetVertexCount())) {
      return {"rejected", "support_index_a_out_of_range", {}};
    }
    if (index_b >= static_cast<std::uint32_t>(proxy_b.GetVertexCount())) {
      return {"rejected", "support_index_b_out_of_range", {}};
    }
    const auto key = (static_cast<std::uint64_t>(index_a) << 32U) | index_b;
    if (!seen.insert(key).second) return {"rejected", "duplicate_support_pair", {}};
    decision.cache.indexA[index] = static_cast<std::uint8_t>(index_a);
    decision.cache.indexB[index] = static_cast<std::uint8_t>(index_b);
  }
  decision.cache.metric = scalar(seed.at("metric_bits"));
  if (!b2IsValid(decision.cache.metric)) return {"rejected", "non_finite_metric", {}};
  if (pairs.size() == 1) {
    decision.outcome = "used";
    return decision;
  }
  std::vector<b2Vec2> support;
  for (const auto& pair : pairs) {
    support.push_back(b2Mul(transform_b, proxy_b.GetVertex(pair.at("index_b").get<int32>())) -
                      b2Mul(transform_a, proxy_a.GetVertex(pair.at("index_a").get<int32>())));
  }
  const float metric2 = support.size() == 2 ? b2Distance(support[0], support[1])
                                             : b2Cross(support[1] - support[0], support[2] - support[0]);
  if (metric2 < 0.5F * decision.cache.metric || 2.0F * decision.cache.metric < metric2) {
    return {"reset", "metric_ratio", {}};
  }
  if (metric2 < b2_epsilon) return {"reset", "metric_too_small", {}};
  decision.outcome = "used";
  return decision;
}

void append_distance(Json& result, const b2DistanceOutput& output, const b2SimplexCache& cache) {
  number(result, "point_a_x", output.pointA.x);
  number(result, "point_a_y", output.pointA.y);
  number(result, "point_b_x", output.pointB.x);
  number(result, "point_b_y", output.pointB.y);
  number(result, "distance", output.distance);
  number(result, "cache_metric", cache.metric);
  label(result, "iterations", std::to_string(output.iterations));
  const auto termination = cache.count == 3 ? "triangle" : output.iterations == 0 ? "nearzerodirection"
                                                   : output.iterations >= 20 ? "iterationlimit"
                                                                            : "duplicatesupport";
  label(result, "termination", termination);
  for (std::uint16_t index = 0; index < cache.count; ++index) {
    label(result, "support_" + std::to_string(index) + "_a", std::to_string(cache.indexA[index]));
    label(result, "support_" + std::to_string(index) + "_b", std::to_string(cache.indexB[index]));
  }
}
