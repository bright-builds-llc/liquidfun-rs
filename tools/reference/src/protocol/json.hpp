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
    std::size_t maximum_items;
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
    const auto resolved_bytes = std::holds_alternative<Node::Array>(node.value) &&
                                !frames_.empty() &&
                                frames_.back().maybe_key == "resolved_bytes";
    const auto maximum_items =
        resolved_bytes ? kMaximumRecordBytes : kMaximumCollectionItems;
    frames_.push_back(
        Frame{std::move(node), {}, std::nullopt, maximum_items});
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
      if (array->size() >= frame.maximum_items) {
        return fail("collection exceeds reviewed limit");
      }
      array->push_back(std::move(node));
      return true;
    }
    auto* object = std::get_if<Node::Object>(&frame.node.value);
    if (object == nullptr || !frame.maybe_key.has_value()) {
      return fail("object value appeared without a member name");
    }
    if (object->size() >= frame.maximum_items) {
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

bool as_bool(const Node& node, std::string_view context) {
  const auto* value = std::get_if<bool>(&node.value);
  if (value == nullptr) {
    throw std::runtime_error(std::string(context) + " must be boolean");
  }
  return *value;
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
