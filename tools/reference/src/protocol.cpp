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

Node decode_record_node(std::string_view record) {
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
  if (!Json::sax_parse(
          payload.begin(), payload.end(), &sax, Json::input_format_t::json,
          true)) {
    throw std::runtime_error(sax.error().empty() ? "parse failed" : sax.error());
  }
  return sax.take_root();
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

MathProbeOperation decode_math_operation(std::string_view value) {
  if (value == "is_valid") return MathProbeOperation::is_valid;
  if (value == "abs") return MathProbeOperation::abs;
  if (value == "min") return MathProbeOperation::min;
  if (value == "max") return MathProbeOperation::max;
  if (value == "clamp") return MathProbeOperation::clamp;
  if (value == "inv_sqrt") return MathProbeOperation::inv_sqrt;
  if (value == "vec_length") return MathProbeOperation::vec_length;
  if (value == "vec_normalize") return MathProbeOperation::vec_normalize;
  if (value == "dot") return MathProbeOperation::dot;
  if (value == "cross") return MathProbeOperation::cross;
  if (value == "mat22_solve") return MathProbeOperation::mat22_solve;
  if (value == "mat33_solve") return MathProbeOperation::mat33_solve;
  if (value == "mat22_inverse") return MathProbeOperation::mat22_inverse;
  if (value == "mat33_sym_inverse") return MathProbeOperation::mat33_sym_inverse;
  if (value == "rotation") return MathProbeOperation::rotation;
  if (value == "transform") return MathProbeOperation::transform;
  if (value == "sweep_transform") return MathProbeOperation::sweep_transform;
  if (value == "sweep_advance") return MathProbeOperation::sweep_advance;
  if (value == "sweep_normalize") return MathProbeOperation::sweep_normalize;
  if (value == "cancellation") return MathProbeOperation::cancellation;
  if (value == "halfway_rounding") return MathProbeOperation::halfway_rounding;
  if (value == "overflow") return MathProbeOperation::overflow;
  if (value == "underflow") return MathProbeOperation::underflow;
  if (value == "fma_witness") return MathProbeOperation::fma_witness;
  throw std::runtime_error("unsupported math probe operation");
}

MathProbePolicyPath decode_math_policy_path(std::string_view value) {
  if (value == "math.branch.is_valid") {
    return MathProbePolicyPath::branch_is_valid;
  }
  if (value == "math.operation.abs") return MathProbePolicyPath::operation_abs;
  if (value == "math.operation.min") return MathProbePolicyPath::operation_min;
  if (value == "math.pass_through.max") return MathProbePolicyPath::pass_through_max;
  if (value == "math.operation.clamp") return MathProbePolicyPath::operation_clamp;
  if (value == "math.operation.inv_sqrt") return MathProbePolicyPath::operation_inv_sqrt;
  if (value == "math.vector.length") return MathProbePolicyPath::vector_length;
  if (value == "math.vector.normalize") return MathProbePolicyPath::vector_normalize;
  if (value == "math.vector.dot") return MathProbePolicyPath::vector_dot;
  if (value == "math.vector.cross") return MathProbePolicyPath::vector_cross;
  if (value == "math.matrix22.solve") return MathProbePolicyPath::matrix22_solve;
  if (value == "math.matrix33.solve") return MathProbePolicyPath::matrix33_solve;
  if (value == "math.matrix22.inverse") return MathProbePolicyPath::matrix22_inverse;
  if (value == "math.matrix33.symmetric_inverse") return MathProbePolicyPath::matrix33_symmetric_inverse;
  if (value == "math.rotation") return MathProbePolicyPath::rotation;
  if (value == "math.transform.operation") return MathProbePolicyPath::transform_operation;
  if (value == "math.transform.steps_32") return MathProbePolicyPath::transform_steps_32;
  if (value == "math.sweep.transform") return MathProbePolicyPath::sweep_transform;
  if (value == "math.sweep.advance_steps_4") return MathProbePolicyPath::sweep_advance_steps_4;
  if (value == "math.sweep.normalize") return MathProbePolicyPath::sweep_normalize;
  if (value == "math.arithmetic.cancellation") return MathProbePolicyPath::arithmetic_cancellation;
  if (value == "math.arithmetic.halfway_rounding") return MathProbePolicyPath::arithmetic_halfway_rounding;
  if (value == "math.arithmetic.overflow") return MathProbePolicyPath::arithmetic_overflow;
  if (value == "math.arithmetic.underflow") return MathProbePolicyPath::arithmetic_underflow;
  if (value == "math.arithmetic.fma_witness") return MathProbePolicyPath::arithmetic_fma_witness;
  throw std::runtime_error("unsupported math probe policy path");
}

Vec2Bits decode_vec2_bits(const Node& node, std::string_view context) {
  const auto& object = as_object(node, context);
  require_members(object, {"x_bits", "y_bits"}, context);
  return {
      as_u32(member(object, "x_bits", context), "x bits"),
      as_u32(member(object, "y_bits", context), "y bits")};
}

Vec3Bits decode_vec3_bits(const Node& node, std::string_view context) {
  const auto& object = as_object(node, context);
  require_members(object, {"x_bits", "y_bits", "z_bits"}, context);
  return {
      as_u32(member(object, "x_bits", context), "x bits"),
      as_u32(member(object, "y_bits", context), "y bits"),
      as_u32(member(object, "z_bits", context), "z bits")};
}

Mat22Bits decode_mat22_bits(const Node& node) {
  const auto& object = as_object(node, "mat22 bits");
  require_members(object, {"first", "second"}, "mat22 bits");
  return {
      decode_vec2_bits(member(object, "first", "mat22 bits"), "first column"),
      decode_vec2_bits(member(object, "second", "mat22 bits"), "second column")};
}

Mat33Bits decode_mat33_bits(const Node& node) {
  const auto& object = as_object(node, "mat33 bits");
  require_members(object, {"first", "second", "third"}, "mat33 bits");
  return {
      decode_vec3_bits(member(object, "first", "mat33 bits"), "first column"),
      decode_vec3_bits(member(object, "second", "mat33 bits"), "second column"),
      decode_vec3_bits(member(object, "third", "mat33 bits"), "third column")};
}

TransformBits decode_transform_bits(const Node& node) {
  const auto& object = as_object(node, "transform bits");
  require_members(object, {"position", "angle_bits"}, "transform bits");
  return {
      decode_vec2_bits(member(object, "position", "transform bits"), "position"),
      as_u32(member(object, "angle_bits", "transform bits"), "angle bits")};
}

SweepBits decode_sweep_bits(const Node& node) {
  const auto& object = as_object(node, "sweep bits");
  require_members(
      object,
      {"local_center", "initial_center", "center", "initial_angle_bits",
       "angle_bits", "initial_fraction_bits"},
      "sweep bits");
  return {
      decode_vec2_bits(member(object, "local_center", "sweep bits"), "local center"),
      decode_vec2_bits(member(object, "initial_center", "sweep bits"), "initial center"),
      decode_vec2_bits(member(object, "center", "sweep bits"), "center"),
      as_u32(member(object, "initial_angle_bits", "sweep bits"), "initial angle bits"),
      as_u32(member(object, "angle_bits", "sweep bits"), "angle bits"),
      as_u32(member(object, "initial_fraction_bits", "sweep bits"), "initial fraction bits")};
}

MathProbeHorizon decode_math_horizon(const Node& node) {
  const auto& object = as_object(node, "math probe horizon");
  const auto& kind = as_string(
      member(object, "kind", "math probe horizon"), "horizon kind");
  if (kind == "operation") {
    require_members(object, {"kind"}, "operation horizon");
    return {};
  }
  if (kind == "scenario_steps") {
    require_members(object, {"kind", "steps"}, "scenario-steps horizon");
    const auto steps = as_u32(
        member(object, "steps", "scenario-steps horizon"), "horizon steps");
    if (steps == 0 || steps > 32) {
      throw std::runtime_error("math probe horizon is outside reviewed bounds");
    }
    return {false, steps};
  }
  throw std::runtime_error("unsupported math probe horizon");
}

MathProbeInput decode_math_input(const Node& node) {
  const auto& object = as_object(node, "math probe input");
  const auto& kind = as_string(
      member(object, "kind", "math probe input"), "math input kind");
  if (kind == "scalar") {
    require_members(object, {"kind", "value_bits"}, "scalar input");
    return ScalarInput{as_u32(member(object, "value_bits", "scalar input"), "value bits")};
  }
  if (kind == "binary") {
    require_members(object, {"kind", "a_bits", "b_bits"}, "binary input");
    return BinaryInput{
        as_u32(member(object, "a_bits", "binary input"), "a bits"),
        as_u32(member(object, "b_bits", "binary input"), "b bits")};
  }
  if (kind == "clamp") {
    require_members(object, {"kind", "value_bits", "low_bits", "high_bits"}, "clamp input");
    return ClampInput{
        as_u32(member(object, "value_bits", "clamp input"), "value bits"),
        as_u32(member(object, "low_bits", "clamp input"), "low bits"),
        as_u32(member(object, "high_bits", "clamp input"), "high bits")};
  }
  if (kind == "vector2") {
    require_members(object, {"kind", "vector"}, "vector2 input");
    return Vector2Input{decode_vec2_bits(member(object, "vector", "vector2 input"), "vector")};
  }
  if (kind == "vector_pair") {
    require_members(object, {"kind", "a", "b"}, "vector-pair input");
    return VectorPairInput{
        decode_vec2_bits(member(object, "a", "vector-pair input"), "a vector"),
        decode_vec2_bits(member(object, "b", "vector-pair input"), "b vector")};
  }
  if (kind == "mat22_solve") {
    require_members(object, {"kind", "matrix", "right"}, "mat22-solve input");
    return Mat22SolveInput{
        decode_mat22_bits(member(object, "matrix", "mat22-solve input")),
        decode_vec2_bits(member(object, "right", "mat22-solve input"), "right vector")};
  }
  if (kind == "mat33_solve") {
    require_members(object, {"kind", "matrix", "right"}, "mat33-solve input");
    return Mat33SolveInput{
        decode_mat33_bits(member(object, "matrix", "mat33-solve input")),
        decode_vec3_bits(member(object, "right", "mat33-solve input"), "right vector")};
  }
  if (kind == "mat22") {
    require_members(object, {"kind", "matrix"}, "mat22 input");
    return Mat22Input{decode_mat22_bits(member(object, "matrix", "mat22 input"))};
  }
  if (kind == "mat33") {
    require_members(object, {"kind", "matrix"}, "mat33 input");
    return Mat33Input{decode_mat33_bits(member(object, "matrix", "mat33 input"))};
  }
  if (kind == "rotation") {
    require_members(object, {"kind", "angle_bits"}, "rotation input");
    return RotationInput{as_u32(member(object, "angle_bits", "rotation input"), "angle bits")};
  }
  if (kind == "transform") {
    require_members(object, {"kind", "left", "right", "point"}, "transform input");
    return TransformInput{
        decode_transform_bits(member(object, "left", "transform input")),
        decode_transform_bits(member(object, "right", "transform input")),
        decode_vec2_bits(member(object, "point", "transform input"), "point")};
  }
  if (kind == "sweep_transform") {
    require_members(object, {"kind", "sweep", "fraction_bits"}, "sweep-transform input");
    return SweepTransformInput{
        decode_sweep_bits(member(object, "sweep", "sweep-transform input")),
        as_u32(member(object, "fraction_bits", "sweep-transform input"), "fraction bits")};
  }
  if (kind == "sweep_advance") {
    require_members(object, {"kind", "sweep", "fractions_bits"}, "sweep-advance input");
    const auto& raw_fractions = as_array(
        member(object, "fractions_bits", "sweep-advance input"), "fractions bits");
    if (raw_fractions.size() > 32) {
      throw std::runtime_error("math probe step collection exceeds reviewed limit");
    }
    std::vector<std::uint32_t> fractions;
    fractions.reserve(raw_fractions.size());
    for (const auto& raw_fraction : raw_fractions) {
      fractions.push_back(as_u32(raw_fraction, "fraction bits"));
    }
    return SweepAdvanceInput{
        decode_sweep_bits(member(object, "sweep", "sweep-advance input")),
        std::move(fractions)};
  }
  if (kind == "sweep") {
    require_members(object, {"kind", "sweep"}, "sweep input");
    return SweepInput{decode_sweep_bits(member(object, "sweep", "sweep input"))};
  }
  if (kind == "cancellation") {
    require_members(object, {"kind", "large_bits", "opposite_bits", "tail_bits"}, "cancellation input");
    return CancellationInput{
        as_u32(member(object, "large_bits", "cancellation input"), "large bits"),
        as_u32(member(object, "opposite_bits", "cancellation input"), "opposite bits"),
        as_u32(member(object, "tail_bits", "cancellation input"), "tail bits")};
  }
  if (kind == "halfway_rounding") {
    require_members(object, {"kind", "even_bits", "odd_bits", "half_ulp_bits"}, "halfway-rounding input");
    return HalfwayRoundingInput{
        as_u32(member(object, "even_bits", "halfway-rounding input"), "even bits"),
        as_u32(member(object, "odd_bits", "halfway-rounding input"), "odd bits"),
        as_u32(member(object, "half_ulp_bits", "halfway-rounding input"), "half ULP bits")};
  }
  if (kind == "scale") {
    require_members(object, {"kind", "value_bits", "factor_bits"}, "scale input");
    return ScaleInput{
        as_u32(member(object, "value_bits", "scale input"), "value bits"),
        as_u32(member(object, "factor_bits", "scale input"), "factor bits")};
  }
  if (kind == "fma_witness") {
    require_members(object, {"kind", "a_bits", "b_bits", "c_bits"}, "FMA-witness input");
    return FmaWitnessInput{
        as_u32(member(object, "a_bits", "FMA-witness input"), "a bits"),
        as_u32(member(object, "b_bits", "FMA-witness input"), "b bits"),
        as_u32(member(object, "c_bits", "FMA-witness input"), "c bits")};
  }
  throw std::runtime_error("unsupported math probe input kind");
}

bool input_matches_operation(
    MathProbeOperation operation,
    const MathProbeInput& input) {
  switch (operation) {
    case MathProbeOperation::is_valid:
    case MathProbeOperation::abs:
    case MathProbeOperation::inv_sqrt:
      return std::holds_alternative<ScalarInput>(input);
    case MathProbeOperation::min:
    case MathProbeOperation::max:
      return std::holds_alternative<BinaryInput>(input);
    case MathProbeOperation::clamp:
      return std::holds_alternative<ClampInput>(input);
    case MathProbeOperation::vec_length:
    case MathProbeOperation::vec_normalize:
      return std::holds_alternative<Vector2Input>(input);
    case MathProbeOperation::dot:
    case MathProbeOperation::cross:
      return std::holds_alternative<VectorPairInput>(input);
    case MathProbeOperation::mat22_solve:
      return std::holds_alternative<Mat22SolveInput>(input);
    case MathProbeOperation::mat33_solve:
      return std::holds_alternative<Mat33SolveInput>(input);
    case MathProbeOperation::mat22_inverse:
      return std::holds_alternative<Mat22Input>(input);
    case MathProbeOperation::mat33_sym_inverse:
      return std::holds_alternative<Mat33Input>(input);
    case MathProbeOperation::rotation:
      return std::holds_alternative<RotationInput>(input);
    case MathProbeOperation::transform:
      return std::holds_alternative<TransformInput>(input);
    case MathProbeOperation::sweep_transform:
      return std::holds_alternative<SweepTransformInput>(input);
    case MathProbeOperation::sweep_advance:
      return std::holds_alternative<SweepAdvanceInput>(input);
    case MathProbeOperation::sweep_normalize:
      return std::holds_alternative<SweepInput>(input);
    case MathProbeOperation::cancellation:
      return std::holds_alternative<CancellationInput>(input);
    case MathProbeOperation::halfway_rounding:
      return std::holds_alternative<HalfwayRoundingInput>(input);
    case MathProbeOperation::overflow:
    case MathProbeOperation::underflow:
      return std::holds_alternative<ScaleInput>(input);
    case MathProbeOperation::fma_witness:
      return std::holds_alternative<FmaWitnessInput>(input);
  }
  return false;
}

MathProbePolicyPath expected_policy_path(
    MathProbeOperation operation,
    const MathProbeHorizon& horizon) {
  switch (operation) {
    case MathProbeOperation::is_valid:
      return MathProbePolicyPath::branch_is_valid;
    case MathProbeOperation::abs: return MathProbePolicyPath::operation_abs;
    case MathProbeOperation::min: return MathProbePolicyPath::operation_min;
    case MathProbeOperation::max: return MathProbePolicyPath::pass_through_max;
    case MathProbeOperation::clamp: return MathProbePolicyPath::operation_clamp;
    case MathProbeOperation::inv_sqrt: return MathProbePolicyPath::operation_inv_sqrt;
    case MathProbeOperation::vec_length: return MathProbePolicyPath::vector_length;
    case MathProbeOperation::vec_normalize: return MathProbePolicyPath::vector_normalize;
    case MathProbeOperation::dot: return MathProbePolicyPath::vector_dot;
    case MathProbeOperation::cross: return MathProbePolicyPath::vector_cross;
    case MathProbeOperation::mat22_solve: return MathProbePolicyPath::matrix22_solve;
    case MathProbeOperation::mat33_solve: return MathProbePolicyPath::matrix33_solve;
    case MathProbeOperation::mat22_inverse: return MathProbePolicyPath::matrix22_inverse;
    case MathProbeOperation::mat33_sym_inverse:
      return MathProbePolicyPath::matrix33_symmetric_inverse;
    case MathProbeOperation::rotation: return MathProbePolicyPath::rotation;
    case MathProbeOperation::transform:
      return horizon.is_operation ? MathProbePolicyPath::transform_operation
                                  : MathProbePolicyPath::transform_steps_32;
    case MathProbeOperation::sweep_transform:
      return MathProbePolicyPath::sweep_transform;
    case MathProbeOperation::sweep_advance:
      return MathProbePolicyPath::sweep_advance_steps_4;
    case MathProbeOperation::sweep_normalize:
      return MathProbePolicyPath::sweep_normalize;
    case MathProbeOperation::cancellation:
      return MathProbePolicyPath::arithmetic_cancellation;
    case MathProbeOperation::halfway_rounding:
      return MathProbePolicyPath::arithmetic_halfway_rounding;
    case MathProbeOperation::overflow:
      return MathProbePolicyPath::arithmetic_overflow;
    case MathProbeOperation::underflow:
      return MathProbePolicyPath::arithmetic_underflow;
    case MathProbeOperation::fma_witness:
      return MathProbePolicyPath::arithmetic_fma_witness;
  }
  throw std::runtime_error("unreachable math probe operation");
}

MathProbeCase decode_math_case(const Node& node) {
  const auto& object = as_object(node, "math probe case");
  require_members(
      object, {"case_id", "operation", "policy_path", "horizon", "input"},
      "math probe case");
  MathProbeCase probe{
      as_string(member(object, "case_id", "math probe case"), "case ID"),
      decode_math_operation(as_string(
          member(object, "operation", "math probe case"), "math operation")),
      decode_math_policy_path(as_string(
          member(object, "policy_path", "math probe case"), "policy path")),
      decode_math_horizon(member(object, "horizon", "math probe case")),
      decode_math_input(member(object, "input", "math probe case"))};
  require_id(probe.case_id, "math probe case ID");
  if (!input_matches_operation(probe.operation, probe.input)) {
    throw std::runtime_error("math probe operation/input mismatch");
  }
  if (probe.policy_path != expected_policy_path(probe.operation, probe.horizon)) {
    throw std::runtime_error("math probe policy path mismatch");
  }
  if (const auto* advance = std::get_if<SweepAdvanceInput>(&probe.input)) {
    if (advance->fractions.empty() ||
        advance->fractions.size() != probe.horizon.steps ||
        probe.horizon.steps != 4) {
      throw std::runtime_error("math probe advance horizon mismatch");
    }
  } else if (!probe.horizon.is_operation) {
    if (!std::holds_alternative<TransformInput>(probe.input) ||
        probe.horizon.steps != 32) {
      throw std::runtime_error("math probe scenario horizon is invalid");
    }
  }
  return probe;
}

std::string_view math_operation_name(MathProbeOperation operation) {
  switch (operation) {
    case MathProbeOperation::is_valid: return "is_valid";
    case MathProbeOperation::abs: return "abs";
    case MathProbeOperation::min: return "min";
    case MathProbeOperation::max: return "max";
    case MathProbeOperation::clamp: return "clamp";
    case MathProbeOperation::inv_sqrt: return "inv_sqrt";
    case MathProbeOperation::vec_length: return "vec_length";
    case MathProbeOperation::vec_normalize: return "vec_normalize";
    case MathProbeOperation::dot: return "dot";
    case MathProbeOperation::cross: return "cross";
    case MathProbeOperation::mat22_solve: return "mat22_solve";
    case MathProbeOperation::mat33_solve: return "mat33_solve";
    case MathProbeOperation::mat22_inverse: return "mat22_inverse";
    case MathProbeOperation::mat33_sym_inverse: return "mat33_sym_inverse";
    case MathProbeOperation::rotation: return "rotation";
    case MathProbeOperation::transform: return "transform";
    case MathProbeOperation::sweep_transform: return "sweep_transform";
    case MathProbeOperation::sweep_advance: return "sweep_advance";
    case MathProbeOperation::sweep_normalize: return "sweep_normalize";
    case MathProbeOperation::cancellation: return "cancellation";
    case MathProbeOperation::halfway_rounding: return "halfway_rounding";
    case MathProbeOperation::overflow: return "overflow";
    case MathProbeOperation::underflow: return "underflow";
    case MathProbeOperation::fma_witness: return "fma_witness";
  }
  throw std::runtime_error("unreachable math probe operation");
}

std::string_view math_policy_path_name(MathProbePolicyPath path) {
  switch (path) {
    case MathProbePolicyPath::branch_is_valid: return "math.branch.is_valid";
    case MathProbePolicyPath::operation_abs: return "math.operation.abs";
    case MathProbePolicyPath::operation_min: return "math.operation.min";
    case MathProbePolicyPath::pass_through_max: return "math.pass_through.max";
    case MathProbePolicyPath::operation_clamp: return "math.operation.clamp";
    case MathProbePolicyPath::operation_inv_sqrt: return "math.operation.inv_sqrt";
    case MathProbePolicyPath::vector_length: return "math.vector.length";
    case MathProbePolicyPath::vector_normalize: return "math.vector.normalize";
    case MathProbePolicyPath::vector_dot: return "math.vector.dot";
    case MathProbePolicyPath::vector_cross: return "math.vector.cross";
    case MathProbePolicyPath::matrix22_solve: return "math.matrix22.solve";
    case MathProbePolicyPath::matrix33_solve: return "math.matrix33.solve";
    case MathProbePolicyPath::matrix22_inverse: return "math.matrix22.inverse";
    case MathProbePolicyPath::matrix33_symmetric_inverse: return "math.matrix33.symmetric_inverse";
    case MathProbePolicyPath::rotation: return "math.rotation";
    case MathProbePolicyPath::transform_operation: return "math.transform.operation";
    case MathProbePolicyPath::transform_steps_32: return "math.transform.steps_32";
    case MathProbePolicyPath::sweep_transform: return "math.sweep.transform";
    case MathProbePolicyPath::sweep_advance_steps_4: return "math.sweep.advance_steps_4";
    case MathProbePolicyPath::sweep_normalize: return "math.sweep.normalize";
    case MathProbePolicyPath::arithmetic_cancellation: return "math.arithmetic.cancellation";
    case MathProbePolicyPath::arithmetic_halfway_rounding: return "math.arithmetic.halfway_rounding";
    case MathProbePolicyPath::arithmetic_overflow: return "math.arithmetic.overflow";
    case MathProbePolicyPath::arithmetic_underflow: return "math.arithmetic.underflow";
    case MathProbePolicyPath::arithmetic_fma_witness: return "math.arithmetic.fma_witness";
  }
  throw std::runtime_error("unreachable math probe policy path");
}

std::string_view math_value_field_name(MathProbeValueField field) {
  switch (field) {
    case MathProbeValueField::value: return "value";
    case MathProbeValueField::x: return "x";
    case MathProbeValueField::y: return "y";
    case MathProbeValueField::z: return "z";
    case MathProbeValueField::length: return "length";
    case MathProbeValueField::sine: return "sine";
    case MathProbeValueField::cosine: return "cosine";
    case MathProbeValueField::position_x: return "position_x";
    case MathProbeValueField::position_y: return "position_y";
    case MathProbeValueField::angle: return "angle";
    case MathProbeValueField::initial_center_x: return "initial_center_x";
    case MathProbeValueField::initial_center_y: return "initial_center_y";
    case MathProbeValueField::initial_angle: return "initial_angle";
    case MathProbeValueField::initial_fraction: return "initial_fraction";
    case MathProbeValueField::left_associated: return "left_associated";
    case MathProbeValueField::right_associated: return "right_associated";
    case MathProbeValueField::even_midpoint: return "even_midpoint";
    case MathProbeValueField::odd_midpoint: return "odd_midpoint";
  }
  throw std::runtime_error("unreachable math probe value field");
}

std::string_view math_discrete_field_name(MathProbeDiscreteField field) {
  switch (field) {
    case MathProbeDiscreteField::predicate: return "predicate";
    case MathProbeDiscreteField::non_zero_determinant: return "non_zero_determinant";
    case MathProbeDiscreteField::normalized: return "normalized";
    case MathProbeDiscreteField::advanced: return "advanced";
  }
  throw std::runtime_error("unreachable math probe discrete field");
}

std::string_view float_class_name(std::uint32_t bits) {
  const auto exponent = bits & 0x7F800000U;
  const auto fraction = bits & 0x007FFFFFU;
  if (exponent == 0) return fraction == 0 ? "zero" : "subnormal";
  if (exponent == 0x7F800000U) return fraction == 0 ? "infinite" : "nan";
  return "normal";
}

}  // namespace

RequestKind decode_request_kind(std::string_view record) {
  const auto root = decode_record_node(record);
  const auto& object = as_object(root, "protocol request");
  const auto& kind = as_string(
      member(object, "record_kind", "protocol request"), "record kind");
  if (kind == "scenario_request") return RequestKind::scenario;
  if (kind == "math_probe_request") return RequestKind::math_probe;
  if (kind == "collision_probe_request") return RequestKind::collision_probe;
  if (kind == "rigid_world_request") return RequestKind::rigid_world;
  throw std::runtime_error("unsupported record kind");
}

MathProbeRequest decode_math_probe_request(std::string_view record) {
  const auto root = decode_record_node(record);
  const auto& object = as_object(root, "math probe request");
  require_members(
      object,
      {"protocol_version", "record_kind", "request_id",
       "scenario_schema_version", "requested_trace_schema_version",
       "tolerance_profile_version", "tolerance_profile_sha256", "scenario"},
      "math probe request");
  if (as_u32(member(object, "protocol_version", "math probe request"),
             "protocol version") != kProtocolVersion ||
      as_string(member(object, "record_kind", "math probe request"),
                "record kind") != "math_probe_request" ||
      as_u32(member(object, "scenario_schema_version", "math probe request"),
             "scenario version") != kScenarioSchemaVersion ||
      as_u32(member(object, "requested_trace_schema_version", "math probe request"),
             "trace version") != kTraceSchemaVersion ||
      as_u32(member(object, "tolerance_profile_version", "math probe request"),
             "tolerance version") != kToleranceProfileVersion) {
    throw std::runtime_error("unsupported math probe protocol version");
  }
  const auto& request_id = as_string(
      member(object, "request_id", "math probe request"), "request ID");
  require_id(request_id, "request ID");
  require_sha256(
      as_string(
          member(object, "tolerance_profile_sha256", "math probe request"),
          "tolerance digest"),
      "tolerance digest");
  const auto& scenario = as_object(
      member(object, "scenario", "math probe request"), "math probe scenario");
  require_members(
      scenario, {"scenario_id", "source", "cases"}, "math probe scenario");
  const auto& scenario_id = as_string(
      member(scenario, "scenario_id", "math probe scenario"), "scenario ID");
  require_id(scenario_id, "scenario ID");
  static_cast<void>(decode_source(member(scenario, "source", "math probe scenario")));
  const auto& raw_cases = as_array(
      member(scenario, "cases", "math probe scenario"), "math probe cases");
  if (raw_cases.empty() || raw_cases.size() > 256) {
    throw std::runtime_error("math probe case count is outside reviewed bounds");
  }
  std::unordered_set<std::string> case_ids;
  std::vector<MathProbeCase> cases;
  cases.reserve(raw_cases.size());
  for (const auto& raw_case : raw_cases) {
    auto probe = decode_math_case(raw_case);
    if (!case_ids.insert(probe.case_id).second) {
      throw std::runtime_error("duplicate math probe case ID");
    }
    cases.push_back(std::move(probe));
  }
  return {request_id, std::move(cases)};
}

ScenarioRequest decode_scenario_request(std::string_view record) {
  const auto root = decode_record_node(record);
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
         ",\"compile_command_sha256\":" + quote(identity.compile_command_sha256) +
         ",\"target_triple\":" + quote(identity.target_triple) +
         ",\"target_cpu\":" + quote(identity.target_cpu) +
         ",\"target_features\":" + quote(identity.target_features) +
         ",\"sdk_or_sysroot\":" + quote(identity.sdk_or_sysroot) +
         ",\"optimization\":" + quote(identity.optimization) +
         ",\"fp_model\":" + quote(identity.fp_model) +
         ",\"fp_contract\":" + quote(identity.fp_contract) +
         ",\"denormal_mode\":" + quote(identity.denormal_mode) +
         ",\"feature_set\":" + quote(identity.feature_set) +
         ",\"os\":" + quote(identity.os) +
         ",\"libc\":" + quote(identity.libc) +
         ",\"libm\":" + quote(identity.libm) +
         ",\"rounding_mode\":" + quote(identity.rounding_mode) +
         ",\"gradual_underflow\":" + (identity.gradual_underflow ? "true" : "false") +
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

std::string encode_math_probe_result(const MathProbeResult& result) {
  std::string output = "{\"case_id\":" + quote(result.case_id) +
                       ",\"operation\":" + quote(math_operation_name(result.operation)) +
                       ",\"policy_path\":" + quote(math_policy_path_name(result.policy_path)) +
                       ",\"horizon\":{\"kind\":" +
                       quote(result.horizon.is_operation ? "operation" : "scenario_steps");
  if (!result.horizon.is_operation) {
    output += ",\"steps\":" + std::to_string(result.horizon.steps);
  }
  output += "},\"values\":[";
  for (std::size_t index = 0; index < result.values.size(); ++index) {
    if (index != 0) output += ',';
    const auto& value = result.values[index];
    output += "{\"field\":" + quote(math_value_field_name(value.field)) +
              ",\"bits\":" + std::to_string(value.bits) +
              ",\"class\":" + quote(float_class_name(value.bits)) +
              ",\"negative\":" +
              ((value.bits & 0x80000000U) != 0 ? "true" : "false") + "}";
  }
  output += "],\"discrete\":[";
  for (std::size_t index = 0; index < result.discrete.size(); ++index) {
    if (index != 0) output += ',';
    const auto& discrete = result.discrete[index];
    output += "{\"field\":" + quote(math_discrete_field_name(discrete.field)) +
              ",\"value\":" + (discrete.value ? "true" : "false") + "}";
  }
  return output + "]}";
}

std::string encode_math_probe_end(
    const MathProbeRequest& request,
    std::uint32_t result_count,
    std::uint64_t reset_epoch) {
  return "{\"protocol_version\":1,\"record_kind\":\"math_probe_end\",\"request_id\":" +
         quote(request.request_id) + ",\"result_count\":" +
         std::to_string(result_count) + ",\"reset_epoch\":" +
         std::to_string(reset_epoch) +
         ",\"reset_verified\":true}";
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

void validate_bounded_json_record(std::string_view record) {
  static_cast<void>(decode_record_node(record));
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
