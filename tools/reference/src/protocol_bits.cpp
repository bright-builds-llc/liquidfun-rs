#include "protocol.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <iomanip>
#include <limits>
#include <sstream>
#include <stdexcept>
#include <utility>

namespace liquidfun::reference {
namespace {

class Sha256 {
 public:
  void update(std::string_view bytes) {
    for (const auto character : bytes) {
      buffer_[buffer_size_++] = static_cast<std::uint8_t>(character);
      ++total_bytes_;
      if (buffer_size_ == buffer_.size()) {
        transform();
        buffer_size_ = 0;
      }
    }
  }

  std::array<std::uint8_t, 32> finish() {
    const auto bit_count = total_bytes_ * 8U;
    buffer_[buffer_size_++] = 0x80U;
    if (buffer_size_ > 56) {
      while (buffer_size_ < buffer_.size()) buffer_[buffer_size_++] = 0;
      transform();
      buffer_size_ = 0;
    }
    while (buffer_size_ < 56) buffer_[buffer_size_++] = 0;
    for (int shift = 56; shift >= 0; shift -= 8) {
      buffer_[buffer_size_++] = static_cast<std::uint8_t>(bit_count >> shift);
    }
    transform();
    std::array<std::uint8_t, 32> digest{};
    for (std::size_t index = 0; index < state_.size(); ++index) {
      for (std::size_t byte = 0; byte < 4; ++byte) {
        digest[index * 4 + byte] = static_cast<std::uint8_t>(
            state_[index] >> (24U - static_cast<unsigned int>(byte * 8)));
      }
    }
    return digest;
  }

 private:
  static std::uint32_t rotate(std::uint32_t value, unsigned int bits) {
    return (value >> bits) | (value << (32U - bits));
  }

  void transform() {
    static constexpr std::array<std::uint32_t, 64> constants{
        0x428a2f98U,0x71374491U,0xb5c0fbcfU,0xe9b5dba5U,0x3956c25bU,0x59f111f1U,0x923f82a4U,0xab1c5ed5U,
        0xd807aa98U,0x12835b01U,0x243185beU,0x550c7dc3U,0x72be5d74U,0x80deb1feU,0x9bdc06a7U,0xc19bf174U,
        0xe49b69c1U,0xefbe4786U,0x0fc19dc6U,0x240ca1ccU,0x2de92c6fU,0x4a7484aaU,0x5cb0a9dcU,0x76f988daU,
        0x983e5152U,0xa831c66dU,0xb00327c8U,0xbf597fc7U,0xc6e00bf3U,0xd5a79147U,0x06ca6351U,0x14292967U,
        0x27b70a85U,0x2e1b2138U,0x4d2c6dfcU,0x53380d13U,0x650a7354U,0x766a0abbU,0x81c2c92eU,0x92722c85U,
        0xa2bfe8a1U,0xa81a664bU,0xc24b8b70U,0xc76c51a3U,0xd192e819U,0xd6990624U,0xf40e3585U,0x106aa070U,
        0x19a4c116U,0x1e376c08U,0x2748774cU,0x34b0bcb5U,0x391c0cb3U,0x4ed8aa4aU,0x5b9cca4fU,0x682e6ff3U,
        0x748f82eeU,0x78a5636fU,0x84c87814U,0x8cc70208U,0x90befffaU,0xa4506cebU,0xbef9a3f7U,0xc67178f2U};
    std::array<std::uint32_t, 64> words{};
    for (std::size_t index = 0; index < 16; ++index) {
      words[index] = (static_cast<std::uint32_t>(buffer_[index * 4]) << 24U) |
                     (static_cast<std::uint32_t>(buffer_[index * 4 + 1]) << 16U) |
                     (static_cast<std::uint32_t>(buffer_[index * 4 + 2]) << 8U) |
                     static_cast<std::uint32_t>(buffer_[index * 4 + 3]);
    }
    for (std::size_t index = 16; index < words.size(); ++index) {
      const auto s0 = rotate(words[index - 15], 7) ^
                      rotate(words[index - 15], 18) ^
                      (words[index - 15] >> 3U);
      const auto s1 = rotate(words[index - 2], 17) ^
                      rotate(words[index - 2], 19) ^
                      (words[index - 2] >> 10U);
      words[index] = words[index - 16] + s0 + words[index - 7] + s1;
    }
    auto [a, b, c, d, e, f, g, h] = state_;
    for (std::size_t index = 0; index < words.size(); ++index) {
      const auto s1 = rotate(e, 6) ^ rotate(e, 11) ^ rotate(e, 25);
      const auto choice = (e & f) ^ ((~e) & g);
      const auto temporary1 = h + s1 + choice + constants[index] + words[index];
      const auto s0 = rotate(a, 2) ^ rotate(a, 13) ^ rotate(a, 22);
      const auto majority = (a & b) ^ (a & c) ^ (b & c);
      const auto temporary2 = s0 + majority;
      h = g; g = f; f = e; e = d + temporary1;
      d = c; c = b; b = a; a = temporary1 + temporary2;
    }
    state_[0] += a; state_[1] += b; state_[2] += c; state_[3] += d;
    state_[4] += e; state_[5] += f; state_[6] += g; state_[7] += h;
  }

  std::array<std::uint32_t, 8> state_{
      0x6a09e667U,0xbb67ae85U,0x3c6ef372U,0xa54ff53aU,
      0x510e527fU,0x9b05688cU,0x1f83d9abU,0x5be0cd19U};
  std::array<std::uint8_t, 64> buffer_{};
  std::size_t buffer_size_ = 0;
  std::uint64_t total_bytes_ = 0;
};

void update_length_prefixed(Sha256& hasher, std::string_view value) {
  std::string length(sizeof(std::size_t), '\0');
  auto remaining = value.size();
  for (std::size_t index = 0; index < length.size(); ++index) {
    length[length.size() - index - 1] = static_cast<char>(remaining & 0xFFU);
    remaining >>= 8U;
  }
  hasher.update(length);
  hasher.update(value);
}

std::string digest_hex(const std::array<std::uint8_t, 32>& digest) {
  std::ostringstream output;
  output << std::hex << std::setfill('0');
  for (const auto byte : digest) {
    output << std::setw(2) << static_cast<unsigned int>(byte);
  }
  return output.str();
}

bool is_lowercase_hex(std::string_view value, std::size_t expected_size) {
  const auto lowercase_hex = [](unsigned char character) {
    return (character >= '0' && character <= '9') ||
           (character >= 'a' && character <= 'f');
  };
  return value.size() == expected_size &&
         std::all_of(value.begin(), value.end(), lowercase_hex);
}

}  // namespace

std::string sha256_hex(std::string_view bytes) {
  Sha256 hasher;
  hasher.update(bytes);
  return digest_hex(hasher.finish());
}

std::string build_identity_sha256(const BuildIdentity& identity) {
  const std::array<std::pair<std::string_view, std::string_view>, 11> fields{{
      {"oracle_revision", identity.oracle_revision},
      {"adapter_revision", identity.adapter_revision},
      {"adapter_content_sha256", identity.adapter_content_sha256},
      {"cmake_preset", identity.cmake_preset},
      {"compiler_id", identity.compiler_id},
      {"compiler_version", identity.compiler_version},
      {"target", identity.target},
      {"build_type", identity.build_type},
      {"effective_compile_flags", identity.effective_compile_flags},
      {"effective_link_flags", identity.effective_link_flags},
      {"sanitizer_mode", identity.sanitizer_mode}}};
  if (!is_lowercase_hex(identity.oracle_revision, 40) ||
      !is_lowercase_hex(identity.adapter_content_sha256, 64)) {
    throw std::runtime_error("build identity revision or digest is invalid");
  }
  Sha256 hasher;
  for (const auto& [name, value] : fields) {
    if (value.empty()) throw std::runtime_error("build identity field is empty");
    update_length_prefixed(hasher, name);
    update_length_prefixed(hasher, value);
  }
  return digest_hex(hasher.finish());
}

std::string trace_payload_sha256(const std::vector<std::string>& records) {
  Sha256 hasher;
  for (const auto& record : records) update_length_prefixed(hasher, record);
  return digest_hex(hasher.finish());
}

float float_from_bits(std::uint32_t bits) {
  const auto negative = (bits & 0x80000000U) != 0;
  const auto exponent = (bits >> 23U) & 0xFFU;
  const auto fraction = bits & 0x7FFFFFU;
  float value = 0.0F;
  if (exponent == 0xFFU) {
    value = fraction == 0 ? std::numeric_limits<float>::infinity()
                          : std::numeric_limits<float>::quiet_NaN();
  } else if (exponent == 0) {
    value = std::ldexp(static_cast<float>(fraction), -149);
  } else {
    value = std::ldexp(
        static_cast<float>((1U << 23U) | fraction),
        static_cast<int>(exponent) - 150);
  }
  return negative ? -value : value;
}

std::uint32_t bits_from_float(float value) {
  const auto sign = std::signbit(value) ? 0x80000000U : 0U;
  const auto magnitude = std::fabs(value);
  if (std::isnan(magnitude)) return sign | 0x7FC00000U;
  if (std::isinf(magnitude)) return sign | 0x7F800000U;
  if (magnitude == 0.0F) return sign;
  int exponent = 0;
  const auto fraction = std::frexp(magnitude, &exponent);
  if (exponent <= -126) {
    return sign | static_cast<std::uint32_t>(std::ldexp(magnitude, 149));
  }
  const auto encoded_exponent = static_cast<std::uint32_t>(exponent + 126);
  const auto significand = static_cast<std::uint32_t>(std::ldexp(fraction, 24));
  return sign | (encoded_exponent << 23U) | (significand - (1U << 23U));
}

}  // namespace liquidfun::reference
