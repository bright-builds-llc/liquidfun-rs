#pragma once

struct TreeQueryCallback {
  const b2DynamicTree* tree;
  Json* output;
  bool QueryCallback(int32 proxy) {
    output->push_back(*static_cast<std::uint32_t*>(tree->GetUserData(proxy)));
    return true;
  }
};

struct TreeRayCallback {
  const b2DynamicTree* tree;
  Json* output;
  float32 RayCastCallback(const b2RayCastInput&, int32 proxy) {
    output->push_back(*static_cast<std::uint32_t*>(tree->GetUserData(proxy)));
    return -1.0F;
  }
};

void execute_tree(const Json& probe, Json& result) {
  const auto& commands = probe.at("input").at("commands");
  if (commands.empty() || commands.size() > kMaximumCommands) throw std::runtime_error("collision command count outside reviewed bounds");
  b2DynamicTree tree;
  std::map<std::uint32_t, int32> proxies;
  std::map<std::uint32_t, std::unique_ptr<std::uint32_t>> payloads;
  for (const auto& command : commands) {
    const auto kind = command.at("kind").get<std::string>();
    const auto maybe_payload = command.contains("payload_id") ? command.at("payload_id").get<std::uint32_t>() : 0;
    if (kind == "create") {
      payloads[maybe_payload] = std::make_unique<std::uint32_t>(maybe_payload);
      proxies[maybe_payload] = tree.CreateProxy(command_aabb(command), payloads[maybe_payload].get());
      label(result, "created", std::to_string(maybe_payload));
    } else if (kind == "move" && proxies.count(maybe_payload) != 0) {
      label(result, "moved", boolean(tree.MoveProxy(proxies[maybe_payload], command_aabb(command),
                                                     vector(command.at("displacement")))));
    } else if (kind == "destroy" && proxies.count(maybe_payload) != 0) {
      tree.DestroyProxy(proxies[maybe_payload]);
      proxies.erase(maybe_payload);
      label(result, "destroyed", std::to_string(maybe_payload));
    } else if (kind == "query") {
      TreeQueryCallback callback{&tree, &payload_ids(result)};
      tree.Query(&callback, command_aabb(command));
    } else if (kind == "ray") {
      b2RayCastInput input{vector(command.at("start")), vector(command.at("end")),
                           scalar(command.at("max_fraction_bits"))};
      TreeRayCallback callback{&tree, &payload_ids(result)};
      tree.RayCast(&callback, input);
    } else if (kind == "metrics") {
      number(result, "area_ratio", tree.GetAreaRatio());
      label(result, "proxy_count", std::to_string(proxies.size()));
      label(result, "height", std::to_string(tree.GetHeight()));
      label(result, "max_balance", std::to_string(tree.GetMaxBalance()));
    } else if (kind == "update_pairs") {
      label(result, "update_pairs", "not_applicable");
    } else if (kind == "touch" || kind == "refilter") {
      label(result, "unsupported_tree_command_payload", std::to_string(maybe_payload));
    } else {
      label(result, "missing_payload", std::to_string(maybe_payload));
    }
  }
  if (probe.at("operation") == "tree_lifecycle") {
    tree.Validate();
    label(result, "tree_valid", "true");
  }
}
struct PairCallback {
  Json* output;
  void AddPair(void* first, void* second) {
    output->push_back(*static_cast<std::uint32_t*>(first));
    output->push_back(*static_cast<std::uint32_t*>(second));
  }
};

void execute_broad_phase(const Json& probe, Json& result) {
  b2BroadPhase broad;
  std::map<std::uint32_t, int32> proxies;
  std::map<std::uint32_t, std::unique_ptr<std::uint32_t>> payloads;
  for (const auto& command : probe.at("input").at("commands")) {
    const auto kind = command.at("kind").get<std::string>();
    const auto maybe_payload = command.contains("payload_id") ? command.at("payload_id").get<std::uint32_t>() : 0;
    if (kind == "create") {
      payloads[maybe_payload] = std::make_unique<std::uint32_t>(maybe_payload);
      proxies[maybe_payload] = broad.CreateProxy(command_aabb(command), payloads[maybe_payload].get());
    } else if (kind == "move" && proxies.count(maybe_payload) != 0) {
      const auto bounds = command_aabb(command);
      const bool moved = !broad.GetFatAABB(proxies[maybe_payload]).Contains(bounds);
      broad.MoveProxy(proxies[maybe_payload], bounds, vector(command.at("displacement")));
      label(result, "moved", boolean(moved));
    } else if (kind == "touch" && proxies.count(maybe_payload) != 0) {
      broad.TouchProxy(proxies[maybe_payload]);
      label(result, "touched", std::to_string(maybe_payload));
    } else if (kind == "destroy" && proxies.count(maybe_payload) != 0) {
      broad.DestroyProxy(proxies[maybe_payload]);
      proxies.erase(maybe_payload);
      label(result, "destroyed", std::to_string(maybe_payload));
    } else if (kind == "refilter" && proxies.count(maybe_payload) != 0) {
      broad.TouchProxy(proxies[maybe_payload]);
      label(result, "refiltered", std::to_string(maybe_payload));
    } else if (kind == "update_pairs") {
      PairCallback callback{&payload_ids(result)};
      broad.UpdatePairs(&callback);
    } else if (kind == "metrics") {
      number(result, "area_ratio", broad.GetTreeQuality());
      label(result, "proxy_count", std::to_string(broad.GetProxyCount()));
      label(result, "height", std::to_string(broad.GetTreeHeight()));
      label(result, "max_balance", std::to_string(broad.GetTreeBalance()));
    } else if (kind == "query" || kind == "ray") {
      label(result, "unsupported_broad_phase_command", "query_or_ray");
    } else {
      label(result, "missing_payload", std::to_string(maybe_payload));
    }
  }
}
