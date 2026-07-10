#include "protocol.hpp"

#include "nlohmann/json.hpp"

#include <algorithm>
#include <iomanip>
#include <istream>
#include <limits>
#include <optional>
#include <ostream>
#include <set>
#include <sstream>
#include <stdexcept>
#include <unordered_set>
#include <utility>
#include <variant>

namespace liquidfun::reference {
namespace {

using Json = nlohmann::json;

struct Node {
  using Array = std::vector<Node>;
  using Object = std::vector<std::pair<std::string, Node>>;
  using Value = std::variant<
      std::nullptr_t,
      bool,
      std::int64_t,
      std::uint64_t,
      std::string,
      Array,
      Object>;

  Value value = nullptr;
};

class BoundedSax final : public nlohmann::json_sax<Json> {
 public:
  bool null() override { return add(Node{nullptr}); }
  bool boolean(bool value) override { return add(Node{value}); }
  bool number_integer(number_integer_t value) override {
    return add(Node{static_cast<std::int64_t>(value)});
  }
  bool number_unsigned(number_unsigned_t value) override {
    return add(Node{static_cast<std::uint64_t>(value)});
  }
  bool number_float(number_float_t, const string_t&) override {
    return fail("floating JSON numbers are not supported");
  }
  bool string(string_t& value) override {
    if (value.size() > kMaximumStringBytes) {
      return fail("string exceeds reviewed limit");
    }
    return add(Node{std::move(value)});
  }
  bool binary(binary_t&) override {
    return fail("binary JSON values are not supported");
  }
  bool start_object(std::size_t) override {
    return start(Node{Node::Object{}});
  }
  bool key(string_t& value) override {
    if (value.size() > kMaximumStringBytes) {
      return fail("object member string exceeds reviewed limit");
    }
    if (frames_.empty() ||
        !std::holds_alternative<Node::Object>(frames_.back().node.value)) {
      return fail("object member appeared outside an object");
    }
    auto& frame = frames_.back();
    if (frame.maybe_key.has_value()) {
      return fail("object member is missing a value");
    }
    if (!frame.keys.insert(value).second) {
      return fail("duplicate member: " + value);
    }
    frame.maybe_key = std::move(value);
    return true;
  }
  bool end_object() override { return finish<Node::Object>("object"); }
  bool start_array(std::size_t) override { return start(Node{Node::Array{}}); }
  bool end_array() override { return finish<Node::Array>("array"); }
  bool parse_error(
      std::size_t position,
      const std::string&,
      const nlohmann::detail::exception& error) override {
    if (error_.empty()) {
      error_ = "parse error at byte " + std::to_string(position) + ": " +
               error.what();
    }
    return false;
  }

  Node take_root() {
    if (!root_.has_value()) {
      throw std::runtime_error(error_.empty() ? "missing JSON value" : error_);
    }
    return std::move(*root_);
  }

  const std::string& error() const { return error_; }

 private:
  struct Frame {
    Node node;
    std::unordered_set<std::string> keys;
    std::optional<std::string> maybe_key;
  };

  bool fail(std::string message) {
    if (error_.empty()) {
      error_ = std::move(message);
    }
    return false;
  }

  bool start(Node node) {
    if (frames_.size() >= kMaximumDepth) {
      return fail("JSON nesting depth exceeds reviewed limit");
    }
    frames_.push_back(Frame{std::move(node), {}, std::nullopt});
    return true;
  }

  template <typename Container>
  bool finish(const std::string& kind) {
    if (frames_.empty() ||
        !std::holds_alternative<Container>(frames_.back().node.value)) {
      return fail("mismatched JSON " + kind + " terminator");
    }
    if (frames_.back().maybe_key.has_value()) {
      return fail("object member is missing a value");
    }
    auto node = std::move(frames_.back().node);
    frames_.pop_back();
    return add(std::move(node));
  }

  bool add(Node node) {
    if (frames_.empty()) {
      if (root_.has_value()) {
        return fail("multiple top-level JSON values");
      }
      root_ = std::move(node);
      return true;
    }
    auto& frame = frames_.back();
    if (auto* array = std::get_if<Node::Array>(&frame.node.value)) {
      if (array->size() >= kMaximumCollectionItems) {
        return fail("collection exceeds reviewed limit");
      }
      array->push_back(std::move(node));
      return true;
    }
    auto* object = std::get_if<Node::Object>(&frame.node.value);
    if (object == nullptr || !frame.maybe_key.has_value()) {
      return fail("object value appeared without a member name");
    }
    if (object->size() >= kMaximumCollectionItems) {
      return fail("collection exceeds reviewed limit");
    }
    object->emplace_back(std::move(*frame.maybe_key), std::move(node));
    frame.maybe_key.reset();
    return true;
  }

  std::vector<Frame> frames_;
  std::optional<Node> root_;
  std::string error_;
};

const Node::Object& as_object(const Node& node, std::string_view context) {
  const auto* object = std::get_if<Node::Object>(&node.value);
  if (object == nullptr) {
    throw std::runtime_error(std::string(context) + " must be an object");
  }
  return *object;
}

const Node::Array& as_array(const Node& node, std::string_view context) {
  const auto* array = std::get_if<Node::Array>(&node.value);
  if (array == nullptr) {
    throw std::runtime_error(std::string(context) + " must be an array");
  }
  return *array;
}

const std::string& as_string(const Node& node, std::string_view context) {
  const auto* value = std::get_if<std::string>(&node.value);
  if (value == nullptr) {
    throw std::runtime_error(std::string(context) + " must be a string");
  }
  return *value;
}

std::uint64_t as_u64(const Node& node, std::string_view context) {
  if (const auto* value = std::get_if<std::uint64_t>(&node.value)) {
    return *value;
  }
  if (const auto* value = std::get_if<std::int64_t>(&node.value);
      value != nullptr && *value >= 0) {
    return static_cast<std::uint64_t>(*value);
  }
  throw std::runtime_error(std::string(context) + " must be unsigned");
}

std::uint32_t as_u32(const Node& node, std::string_view context) {
  const auto value = as_u64(node, context);
  if (value > std::numeric_limits<std::uint32_t>::max()) {
    throw std::runtime_error(std::string(context) + " exceeds u32");
  }
  return static_cast<std::uint32_t>(value);
}

const Node& member(
    const Node::Object& object,
    std::string_view name,
    std::string_view context) {
  const auto found = std::find_if(
      object.begin(), object.end(), [name](const auto& entry) {
        return entry.first == name;
      });
  if (found == object.end()) {
    throw std::runtime_error(
        std::string(context) + " is missing member " + std::string(name));
  }
  return found->second;
}

void require_members(
    const Node::Object& object,
    std::initializer_list<std::string_view> allowed,
    std::string_view context) {
  for (const auto& [name, value] : object) {
    static_cast<void>(value);
    if (std::find(allowed.begin(), allowed.end(), name) == allowed.end()) {
      throw std::runtime_error(
          std::string(context) + " contains unknown member " + name);
    }
  }
}

bool is_valid_id(std::string_view value) {
  if (value.empty() || value.size() > kMaximumIdBytes) {
    return false;
  }
  const auto valid_first = [](unsigned char character) {
    return (character >= 'a' && character <= 'z') ||
           (character >= '0' && character <= '9');
  };
  const auto valid_rest = [valid_first](unsigned char character) {
    return valid_first(character) || character == '.' || character == '_' ||
           character == '-';
  };
  return valid_first(static_cast<unsigned char>(value.front())) &&
         std::all_of(value.begin() + 1, value.end(), valid_rest);
}

void require_id(std::string_view value, std::string_view context) {
  if (!is_valid_id(value)) {
    throw std::runtime_error(std::string(context) + " is not a valid ID");
  }
}

void require_sha256(std::string_view value, std::string_view context) {
  const auto lowercase_hex = [](unsigned char character) {
    return (character >= '0' && character <= '9') ||
           (character >= 'a' && character <= 'f');
  };
  if (value.size() != 64 || !std::all_of(value.begin(), value.end(), lowercase_hex)) {
    throw std::runtime_error(std::string(context) + " is not a SHA-256 digest");
  }
}

std::string quote(std::string_view value) {
  std::ostringstream output;
  output << '"';
  for (const auto character : value) {
    const auto byte = static_cast<unsigned char>(character);
    switch (character) {
      case '"': output << "\\\""; break;
      case '\\': output << "\\\\"; break;
      case '\b': output << "\\b"; break;
      case '\f': output << "\\f"; break;
      case '\n': output << "\\n"; break;
      case '\r': output << "\\r"; break;
      case '\t': output << "\\t"; break;
      default:
        if (byte < 0x20U) {
          output << "\\u" << std::hex << std::setw(4) << std::setfill('0')
                 << static_cast<unsigned int>(byte) << std::dec;
        } else {
          output << character;
        }
    }
  }
  output << '"';
  return output.str();
}

ScenarioSource decode_source(const Node& node) {
  const auto& object = as_object(node, "scenario source");
  const auto& kind = as_string(member(object, "kind", "scenario source"), "source kind");
  if (kind == "named") {
    require_members(object, {"kind", "name"}, "named source");
    const auto& name = as_string(member(object, "name", "named source"), "source name");
    if (name.find_first_not_of(" \t\r\n") == std::string::npos) {
      throw std::runtime_error("named source must not be blank");
    }
    return ScenarioSource{ScenarioSourceKind::named, name, {}, 0, 0};
  }
  if (kind == "seeded") {
    require_members(
        object,
        {"kind", "generator_id", "generator_version", "seed"},
        "seeded source");
    const auto& generator_id = as_string(
        member(object, "generator_id", "seeded source"), "generator ID");
    const auto generator_version = as_u32(
        member(object, "generator_version", "seeded source"),
        "generator version");
    if (generator_id.find_first_not_of(" \t\r\n") == std::string::npos ||
        generator_version == 0) {
      throw std::runtime_error("seeded source is invalid");
    }
    return ScenarioSource{
        ScenarioSourceKind::seeded,
        {},
        generator_id,
        generator_version,
        as_u64(member(object, "seed", "seeded source"), "seed")};
  }
  throw std::runtime_error("unsupported source kind");
}

StepCommand decode_command(const Node& node) {
  const auto& object = as_object(node, "step command");
  require_members(
      object,
      {"kind", "command_id", "timestep_bits", "velocity_iterations",
       "position_iterations", "particle_iterations"},
      "step command");
  if (as_string(member(object, "kind", "step command"), "command kind") != "step") {
    throw std::runtime_error("unsupported command kind");
  }
  StepCommand command{
      as_string(member(object, "command_id", "step command"), "command ID"),
      as_u32(member(object, "timestep_bits", "step command"), "timestep bits"),
      as_u32(member(object, "velocity_iterations", "step command"), "velocity iterations"),
      as_u32(member(object, "position_iterations", "step command"), "position iterations"),
      as_u32(member(object, "particle_iterations", "step command"), "particle iterations")};
  require_id(command.command_id, "command ID");
  for (const auto iterations : {command.velocity_iterations,
                                command.position_iterations,
                                command.particle_iterations}) {
    if (iterations == 0 || iterations > 255) {
      throw std::runtime_error("solver iterations are outside reviewed bounds");
    }
  }
  return command;
}

CheckpointRequest decode_checkpoint(const Node& node) {
  const auto& object = as_object(node, "checkpoint");
  require_members(
      object,
      {"checkpoint_id", "after_command_id", "phase", "observables"},
      "checkpoint");
  CheckpointRequest checkpoint{
      as_string(member(object, "checkpoint_id", "checkpoint"), "checkpoint ID"),
      as_string(member(object, "after_command_id", "checkpoint"), "command reference"),
      as_string(member(object, "phase", "checkpoint"), "checkpoint phase"),
      {}};
  require_id(checkpoint.checkpoint_id, "checkpoint ID");
  require_id(checkpoint.after_command_id, "checkpoint command reference");
  if (checkpoint.phase.empty()) {
    throw std::runtime_error("checkpoint phase must not be empty");
  }
  const auto& observables = as_array(
      member(object, "observables", "checkpoint"), "checkpoint observables");
  if (observables.size() > kMaximumObservableItems) {
    throw std::runtime_error("observable collection exceeds reviewed limit");
  }
  std::set<Observable> unique;
  for (const auto& observable_node : observables) {
    const auto& value = as_string(observable_node, "observable");
    const auto observable = value == "world_counts"
                                ? Observable::world_counts
                                : value == "simulation_time"
                                      ? Observable::simulation_time
                                      : throw std::runtime_error("unsupported observable");
    if (!unique.insert(observable).second) {
      throw std::runtime_error("duplicate observable");
    }
    checkpoint.observables.push_back(observable);
  }
  return checkpoint;
}

ScenarioV1 decode_scenario(const Node& node) {
  const auto& object = as_object(node, "scenario");
  require_members(
      object,
      {"scenario_id", "source", "gravity_x_bits", "gravity_y_bits", "entities",
       "commands", "checkpoints"},
      "scenario");
  ScenarioV1 scenario{
      as_string(member(object, "scenario_id", "scenario"), "scenario ID"),
      decode_source(member(object, "source", "scenario")),
      as_u32(member(object, "gravity_x_bits", "scenario"), "gravity x bits"),
      as_u32(member(object, "gravity_y_bits", "scenario"), "gravity y bits"),
      {},
      {}};
  require_id(scenario.scenario_id, "scenario ID");
  if (!as_array(member(object, "entities", "scenario"), "entities").empty()) {
    throw std::runtime_error("phase-2 entities must be empty");
  }
  const auto& commands = as_array(member(object, "commands", "scenario"), "commands");
  if (commands.empty()) {
    throw std::runtime_error("scenario must contain a command");
  }
  std::unordered_set<std::string> command_ids;
  for (const auto& command_node : commands) {
    auto command = decode_command(command_node);
    if (!command_ids.insert(command.command_id).second) {
      throw std::runtime_error("duplicate command ID");
    }
    scenario.commands.push_back(std::move(command));
  }
  const auto& checkpoints = as_array(
      member(object, "checkpoints", "scenario"), "checkpoints");
  std::unordered_set<std::string> checkpoint_ids;
  std::size_t previous_command_index = 0;
  for (const auto& checkpoint_node : checkpoints) {
    auto checkpoint = decode_checkpoint(checkpoint_node);
    if (!checkpoint_ids.insert(checkpoint.checkpoint_id).second) {
      throw std::runtime_error("duplicate checkpoint ID");
    }
    const auto command = std::find_if(
        scenario.commands.begin(), scenario.commands.end(),
        [&checkpoint](const auto& candidate) {
          return candidate.command_id == checkpoint.after_command_id;
        });
    if (command == scenario.commands.end()) {
      throw std::runtime_error("checkpoint command reference is unknown");
    }
    const auto command_index = static_cast<std::size_t>(
        std::distance(scenario.commands.begin(), command));
    if (!scenario.checkpoints.empty() && command_index < previous_command_index) {
      throw std::runtime_error("checkpoint command references are out of order");
    }
    previous_command_index = command_index;
    scenario.checkpoints.push_back(std::move(checkpoint));
  }
  return scenario;
}

std::string encode_source(const ScenarioSource& source) {
  if (source.kind == ScenarioSourceKind::named) {
    return "{\"kind\":\"named\",\"name\":" + quote(source.name) + "}";
  }
  return "{\"kind\":\"seeded\",\"generator_id\":" +
         quote(source.generator_id) + ",\"generator_version\":" +
         std::to_string(source.generator_version) + ",\"seed\":" +
         std::to_string(source.seed) + "}";
}

std::string encode_world_counts(const WorldCounts& counts) {
  return "{\"bodies\":" + std::to_string(counts.bodies) +
         ",\"fixtures\":" + std::to_string(counts.fixtures) +
         ",\"joints\":" + std::to_string(counts.joints) +
         ",\"contacts\":" + std::to_string(counts.contacts) +
         ",\"particle_systems\":" + std::to_string(counts.particle_systems) +
         ",\"particle_groups\":" + std::to_string(counts.particle_groups) +
         ",\"particles\":" + std::to_string(counts.particles) + "}";
}

}  // namespace

ScenarioRequest decode_scenario_request(std::string_view record) {
  if (record.size() > kMaximumRecordBytes) {
    throw std::runtime_error("record exceeds reviewed byte limit");
  }
  if (record.empty() || record.back() != '\n') {
    throw std::runtime_error("record must end with exactly one newline");
  }
  if (record.substr(0, record.size() - 1).find('\n') != std::string_view::npos) {
    throw std::runtime_error("record contains more than one newline");
  }
  const auto payload = record.substr(0, record.size() - 1);
  BoundedSax sax;
  if (!Json::sax_parse(payload.begin(), payload.end(), &sax, Json::input_format_t::json, true)) {
    throw std::runtime_error(sax.error().empty() ? "parse failed" : sax.error());
  }
  const auto root = sax.take_root();
  const auto& object = as_object(root, "scenario request");
  require_members(
      object,
      {"protocol_version", "record_kind", "request_id", "scenario_schema_version",
       "requested_trace_schema_version", "tolerance_profile_version",
       "tolerance_profile_sha256", "scenario"},
      "scenario request");
  if (as_u32(member(object, "protocol_version", "scenario request"), "protocol version") != kProtocolVersion) {
    throw std::runtime_error("unsupported protocol version");
  }
  if (as_string(member(object, "record_kind", "scenario request"), "record kind") != "scenario_request") {
    throw std::runtime_error("unsupported record kind");
  }
  const auto request_id =
      as_string(member(object, "request_id", "scenario request"), "request ID");
  require_id(request_id, "request ID");
  if (as_u32(member(object, "scenario_schema_version", "scenario request"), "scenario version") != kScenarioSchemaVersion ||
      as_u32(member(object, "requested_trace_schema_version", "scenario request"), "trace version") != kTraceSchemaVersion ||
      as_u32(member(object, "tolerance_profile_version", "scenario request"), "tolerance version") != kToleranceProfileVersion) {
    throw std::runtime_error("unsupported schema or tolerance version");
  }
  ScenarioRequest request{
      request_id,
      as_string(member(object, "tolerance_profile_sha256", "scenario request"), "tolerance digest"),
      decode_scenario(member(object, "scenario", "scenario request"))};
  require_sha256(request.tolerance_profile_sha256, "tolerance digest");
  return request;
}

std::string encode_scenario(const ScenarioV1& scenario) {
  std::string output = "{\"scenario_id\":" + quote(scenario.scenario_id) +
                       ",\"source\":" + encode_source(scenario.source) +
                       ",\"gravity_x_bits\":" + std::to_string(scenario.gravity_x_bits) +
                       ",\"gravity_y_bits\":" + std::to_string(scenario.gravity_y_bits) +
                       ",\"entities\":[],\"commands\":[";
  for (std::size_t index = 0; index < scenario.commands.size(); ++index) {
    if (index != 0) output += ',';
    const auto& command = scenario.commands[index];
    output += "{\"kind\":\"step\",\"command_id\":" + quote(command.command_id) +
              ",\"timestep_bits\":" + std::to_string(command.timestep_bits) +
              ",\"velocity_iterations\":" + std::to_string(command.velocity_iterations) +
              ",\"position_iterations\":" + std::to_string(command.position_iterations) +
              ",\"particle_iterations\":" + std::to_string(command.particle_iterations) + "}";
  }
  output += "],\"checkpoints\":[";
  for (std::size_t index = 0; index < scenario.checkpoints.size(); ++index) {
    if (index != 0) output += ',';
    const auto& checkpoint = scenario.checkpoints[index];
    output += "{\"checkpoint_id\":" + quote(checkpoint.checkpoint_id) +
              ",\"after_command_id\":" + quote(checkpoint.after_command_id) +
              ",\"phase\":" + quote(checkpoint.phase) + ",\"observables\":[";
    for (std::size_t observable = 0; observable < checkpoint.observables.size(); ++observable) {
      if (observable != 0) output += ',';
      output += checkpoint.observables[observable] == Observable::world_counts
                    ? "\"world_counts\""
                    : "\"simulation_time\"";
    }
    output += "]}";
  }
  return output + "]}";
}

std::string encode_scenario_request(const ScenarioRequest& request) {
  return "{\"protocol_version\":1,\"record_kind\":\"scenario_request\",\"request_id\":" +
         quote(request.request_id) +
         ",\"scenario_schema_version\":1,\"requested_trace_schema_version\":1,\"tolerance_profile_version\":1,\"tolerance_profile_sha256\":" +
         quote(request.tolerance_profile_sha256) + ",\"scenario\":" +
         encode_scenario(request.scenario) + "}\n";
}

std::string encode_handshake(const BuildIdentity& identity) {
  const auto identity_sha256 = build_identity_sha256(identity);
  return "{\"protocol_version\":1,\"record_kind\":\"handshake\",\"supported_scenario_versions\":[1],\"supported_trace_versions\":[1],\"supported_tolerance_versions\":[1],\"build_identity\":{\"oracle_revision\":" +
         quote(identity.oracle_revision) + ",\"adapter_revision\":" + quote(identity.adapter_revision) +
         ",\"adapter_content_sha256\":" + quote(identity.adapter_content_sha256) +
         ",\"cmake_preset\":" + quote(identity.cmake_preset) +
         ",\"compiler_id\":" + quote(identity.compiler_id) +
         ",\"compiler_version\":" + quote(identity.compiler_version) +
         ",\"target\":" + quote(identity.target) +
         ",\"build_type\":" + quote(identity.build_type) +
         ",\"effective_compile_flags\":" + quote(identity.effective_compile_flags) +
         ",\"effective_link_flags\":" + quote(identity.effective_link_flags) +
         ",\"sanitizer_mode\":" + quote(identity.sanitizer_mode) +
         "},\"identity_sha256\":" + quote(identity_sha256) + "}";
}

std::string encode_trace_begin(
    const ScenarioRequest& request,
    std::string_view scenario_sha256,
    std::string_view identity_sha256) {
  return "{\"protocol_version\":1,\"record_kind\":\"trace_begin\",\"request_id\":" + quote(request.request_id) +
         ",\"trace_schema_version\":1,\"scenario_id\":" + quote(request.scenario.scenario_id) +
         ",\"scenario_sha256\":" + quote(scenario_sha256) + ",\"source\":" + encode_source(request.scenario.source) +
         ",\"tolerance_profile_version\":1,\"tolerance_profile_sha256\":" + quote(request.tolerance_profile_sha256) +
         ",\"engine_kind\":\"cpp_oracle\",\"identity_sha256\":" + quote(identity_sha256) + "}";
}

std::string encode_checkpoint(
    const ScenarioRequest& request,
    const CheckpointRequest& checkpoint,
    std::uint32_t ordinal,
    std::uint32_t simulation_time_bits,
    const WorldCounts& counts,
    std::string_view identity_sha256) {
  return "{\"protocol_version\":1,\"record_kind\":\"checkpoint\",\"request_id\":" + quote(request.request_id) +
         ",\"checkpoint_id\":" + quote(checkpoint.checkpoint_id) +
         ",\"ordinal\":" + std::to_string(ordinal) + ",\"phase\":" + quote(checkpoint.phase) +
         ",\"simulation_time_bits\":" + std::to_string(simulation_time_bits) +
         ",\"world_counts\":" + encode_world_counts(counts) +
         ",\"identity_sha256\":" + quote(identity_sha256) + "}";
}

std::string encode_trace_end(
    const ScenarioRequest& request,
    std::uint32_t checkpoint_count,
    std::string_view trace_payload_sha256,
    std::uint64_t reset_epoch,
    bool reset_verified,
    std::string_view identity_sha256) {
  return "{\"protocol_version\":1,\"record_kind\":\"trace_end\",\"request_id\":" + quote(request.request_id) +
         ",\"checkpoint_count\":" + std::to_string(checkpoint_count) +
         ",\"trace_payload_sha256\":" + quote(trace_payload_sha256) +
         ",\"reset_epoch\":" + std::to_string(reset_epoch) +
         ",\"reset_verified\":" + (reset_verified ? "true" : "false") +
         ",\"identity_sha256\":" + quote(identity_sha256) + "}";
}

bool read_bounded_record(std::istream& input, std::string& record) {
  record.clear();
  char byte = 0;
  while (input.get(byte)) {
    if (record.size() == kMaximumRecordBytes) {
      throw std::runtime_error("input record exceeds reviewed byte limit");
    }
    record.push_back(byte);
    if (byte == '\n') return true;
  }
  if (!input.eof()) {
    throw std::runtime_error("failed while reading protocol stdin");
  }
  return !record.empty();
}

void write_record(std::ostream& output, std::string_view record) {
  if (record.size() + 1 > kMaximumRecordBytes) {
    throw std::runtime_error("output record exceeds reviewed byte limit");
  }
  output << record << '\n';
  output.flush();
  if (!output) throw std::runtime_error("failed to write protocol record");
}

}  // namespace liquidfun::reference
