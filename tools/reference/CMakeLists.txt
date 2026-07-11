cmake_minimum_required(VERSION 3.25)

project(liquidfun_reference LANGUAGES C CXX)

include(CheckCXXCompilerFlag)

# CMake 4 no longer accepts the upstream project's 2.8 policy baseline unless
# the caller supplies a modern compatibility floor. Keep that adaptation in
# this wrapper so the pinned oracle remains byte-for-byte read-only.
set(
  CMAKE_POLICY_VERSION_MINIMUM
  "3.5"
  CACHE STRING
  "Minimum policy version used by the legacy LiquidFun CMake project"
  FORCE
)

set(BOX2D_INSTALL OFF CACHE BOOL "Do not install the reference library" FORCE)
set(BOX2D_INSTALL_DOC OFF CACHE BOOL "Do not install upstream documentation" FORCE)
set(BOX2D_BUILD_SHARED OFF CACHE BOOL "Build only the static reference library" FORCE)
set(BOX2D_BUILD_STATIC ON CACHE BOOL "Build the static reference library" FORCE)
set(BOX2D_BUILD_EXAMPLES OFF CACHE BOOL "Do not build the upstream testbed" FORCE)
set(BOX2D_BUILD_UNITTESTS OFF CACHE BOOL "Do not build upstream tests in oracle presets" FORCE)
set(BOX2D_CODE_COVERAGE OFF CACHE BOOL "Do not instrument canonical oracle builds" FORCE)

set(
  REFERENCE_REQUIRED_FP_OPTIONS
  -fno-fast-math
  -ffp-model=precise
  -ffp-contract=off
  -fsigned-zeros
  -fhonor-nans
  -fhonor-infinities
  -fdenormal-fp-math=ieee
  -fdenormal-fp-math-fp32=ieee
)
set(REFERENCE_SUPPORTED_FP_OPTIONS "")
set(REFERENCE_MISSING_FP_OPTIONS "")
if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
  foreach(REFERENCE_FP_OPTION IN LISTS REFERENCE_REQUIRED_FP_OPTIONS)
    string(MAKE_C_IDENTIFIER "${REFERENCE_FP_OPTION}" REFERENCE_FP_OPTION_ID)
    set(REFERENCE_FP_CHECK "REFERENCE_SUPPORTS_${REFERENCE_FP_OPTION_ID}")
    check_cxx_compiler_flag("${REFERENCE_FP_OPTION}" "${REFERENCE_FP_CHECK}")
    if(${REFERENCE_FP_CHECK})
      list(APPEND REFERENCE_SUPPORTED_FP_OPTIONS "${REFERENCE_FP_OPTION}")
    else()
      list(APPEND REFERENCE_MISSING_FP_OPTIONS "${REFERENCE_FP_OPTION}")
    endif()
  endforeach()
else()
  set(REFERENCE_MISSING_FP_OPTIONS ${REFERENCE_REQUIRED_FP_OPTIONS})
endif()

string(TOUPPER "${CMAKE_BUILD_TYPE}" REFERENCE_BUILD_TYPE_UPPER)
set(REFERENCE_CONFIG_COMPILE_FLAGS_VARIABLE "CMAKE_CXX_FLAGS_${REFERENCE_BUILD_TYPE_UPPER}")
set(
  REFERENCE_USER_COMPILE_FLAGS
  "${CMAKE_CXX_FLAGS} ${${REFERENCE_CONFIG_COMPILE_FLAGS_VARIABLE}}"
)
set(
  REFERENCE_FORBIDDEN_FP_PATTERN
  "(^|[ ;])(-ffast-math|-Ofast|-fassociative-math|-freciprocal-math|-funsafe-math-optimizations|-march=native|-mcpu=native)($|[ ;])"
)
set(REFERENCE_HAS_FORBIDDEN_FP_FLAGS FALSE)
if(REFERENCE_USER_COMPILE_FLAGS MATCHES "${REFERENCE_FORBIDDEN_FP_PATTERN}")
  set(REFERENCE_HAS_FORBIDDEN_FP_FLAGS TRUE)
endif()
set(REFERENCE_D1_CANDIDATE FALSE)
if(
  CMAKE_CXX_COMPILER_ID STREQUAL "Clang"
  AND CMAKE_CXX_COMPILER_VERSION VERSION_EQUAL "22.1.8"
  AND CMAKE_SYSTEM_NAME STREQUAL "Linux"
  AND CMAKE_SYSTEM_PROCESSOR MATCHES "^(x86_64|amd64|AMD64)$"
)
  set(REFERENCE_D1_CANDIDATE TRUE)
endif()
if(REFERENCE_D1_CANDIDATE AND REFERENCE_MISSING_FP_OPTIONS)
  message(
    FATAL_ERROR
    "canonical D1 compiler lacks required floating semantics: ${REFERENCE_MISSING_FP_OPTIONS}"
  )
endif()
if(REFERENCE_D1_CANDIDATE AND REFERENCE_HAS_FORBIDDEN_FP_FLAGS)
  message(FATAL_ERROR "canonical D1 configuration contains forbidden floating/native flags")
endif()
if(REFERENCE_D1_CANDIDATE)
  set(REFERENCE_EVIDENCE_TIER "d1_canonical")
elseif(REFERENCE_HAS_FORBIDDEN_FP_FLAGS)
  set(REFERENCE_EVIDENCE_TIER "d3_exploratory")
else()
  set(REFERENCE_EVIDENCE_TIER "d2_supported")
endif()

add_subdirectory(
  "${CMAKE_CURRENT_LIST_DIR}/../../third_party/liquidfun/liquidfun/Box2D"
  "${CMAKE_BINARY_DIR}/upstream"
)
target_compile_options(Box2D PRIVATE ${REFERENCE_SUPPORTED_FP_OPTIONS})

set(REFERENCE_UPSTREAM_COMPATIBILITY_OPTIONS "")
if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
  # The read-only 2014 oracle trips two modern Clang warnings under its own
  # -Werror policy in ordinary and sanitizer builds. Keep warnings visible
  # without mutating upstream sources or weakening repository-authored code.
  set(
    REFERENCE_UPSTREAM_COMPATIBILITY_OPTIONS
    -Wno-error=unused-result
    -Wno-error=nontrivial-memcall
  )
  target_compile_options(
    Box2D
    PRIVATE ${REFERENCE_UPSTREAM_COMPATIBILITY_OPTIONS}
  )
endif()

include(CTest)

foreach(
  REQUIRED_IDENTITY_VARIABLE
  IN ITEMS
    LIQUIDFUN_EXPECTED_ORACLE_REVISION
    LIQUIDFUN_EXPECTED_ADAPTER_SHA256
)
  if(
    NOT DEFINED ${REQUIRED_IDENTITY_VARIABLE}
    OR "${${REQUIRED_IDENTITY_VARIABLE}}" STREQUAL ""
  )
    message(
      FATAL_ERROR
      "${REQUIRED_IDENTITY_VARIABLE} must be passed by the verified xtask configure command"
    )
  endif()
endforeach()

string(LENGTH "${LIQUIDFUN_EXPECTED_ORACLE_REVISION}" REFERENCE_REVISION_LENGTH)
if(
  NOT REFERENCE_REVISION_LENGTH EQUAL 40
  OR NOT "${LIQUIDFUN_EXPECTED_ORACLE_REVISION}" MATCHES "^[0-9a-f]+$"
)
  message(FATAL_ERROR "LIQUIDFUN_EXPECTED_ORACLE_REVISION must be full lowercase 40-hex")
endif()
string(LENGTH "${LIQUIDFUN_EXPECTED_ADAPTER_SHA256}" REFERENCE_DIGEST_LENGTH)
if(
  NOT REFERENCE_DIGEST_LENGTH EQUAL 64
  OR NOT "${LIQUIDFUN_EXPECTED_ADAPTER_SHA256}" MATCHES "^[0-9a-f]+$"
)
  message(FATAL_ERROR "LIQUIDFUN_EXPECTED_ADAPTER_SHA256 must be lowercase 64-hex")
endif()

add_library(
  liquidfun-reference-protocol
  STATIC
  src/math_probe.cpp
  src/protocol.cpp
  src/protocol_bits.cpp
  src/oracle_adapter.cpp
)
target_compile_features(liquidfun-reference-protocol PUBLIC cxx_std_17)
target_include_directories(
  liquidfun-reference-protocol
  PUBLIC "${CMAKE_CURRENT_LIST_DIR}/src"
  PRIVATE "${CMAKE_CURRENT_LIST_DIR}/vendor"
)
target_include_directories(
  liquidfun-reference-protocol
  SYSTEM PRIVATE
  "${CMAKE_CURRENT_LIST_DIR}/../../third_party/liquidfun/liquidfun/Box2D"
)
target_link_libraries(liquidfun-reference-protocol PRIVATE Box2D)

set(REFERENCE_ADAPTER_INPUT_MANIFEST "${CMAKE_CURRENT_LIST_DIR}/adapter-inputs.txt")
file(STRINGS "${REFERENCE_ADAPTER_INPUT_MANIFEST}" REFERENCE_ADAPTER_INPUT_LINES)
set(REFERENCE_ADAPTER_SOURCES "")
foreach(REFERENCE_ADAPTER_INPUT_LINE IN LISTS REFERENCE_ADAPTER_INPUT_LINES)
  string(STRIP "${REFERENCE_ADAPTER_INPUT_LINE}" REFERENCE_ADAPTER_SOURCE)
  if(
    NOT REFERENCE_ADAPTER_SOURCE
    OR REFERENCE_ADAPTER_SOURCE MATCHES "^#"
  )
    continue()
  endif()
  if(
    IS_ABSOLUTE "${REFERENCE_ADAPTER_SOURCE}"
    OR REFERENCE_ADAPTER_SOURCE MATCHES "(^|/)\\.\\.(/|$)"
  )
    message(FATAL_ERROR "invalid adapter input: ${REFERENCE_ADAPTER_SOURCE}")
  endif()
  list(APPEND REFERENCE_ADAPTER_SOURCES "${REFERENCE_ADAPTER_SOURCE}")
endforeach()
if(NOT REFERENCE_ADAPTER_SOURCES)
  message(FATAL_ERROR "adapter input manifest is empty")
endif()
set(REFERENCE_ADAPTER_DIGEST_INPUT "")
set(REFERENCE_ADAPTER_DEPENDS "${REFERENCE_ADAPTER_INPUT_MANIFEST}")
foreach(REFERENCE_ADAPTER_SOURCE IN LISTS REFERENCE_ADAPTER_SOURCES)
  file(
    REAL_PATH
    "${CMAKE_CURRENT_LIST_DIR}/../../${REFERENCE_ADAPTER_SOURCE}"
    REFERENCE_ADAPTER_PATH
  )
  list(APPEND REFERENCE_ADAPTER_DEPENDS "${REFERENCE_ADAPTER_PATH}")
  file(SHA256 "${REFERENCE_ADAPTER_PATH}" REFERENCE_ADAPTER_SOURCE_SHA256)
  string(
    APPEND
    REFERENCE_ADAPTER_DIGEST_INPUT
    "${REFERENCE_ADAPTER_SOURCE}=${REFERENCE_ADAPTER_SOURCE_SHA256}\n"
  )
endforeach()
string(SHA256 REFERENCE_CONTENT_SHA256 "${REFERENCE_ADAPTER_DIGEST_INPUT}")
if(
  NOT "${REFERENCE_CONTENT_SHA256}" STREQUAL "${LIQUIDFUN_EXPECTED_ADAPTER_SHA256}"
)
  message(
    FATAL_ERROR
    "adapter digest mismatch: expected ${LIQUIDFUN_EXPECTED_ADAPTER_SHA256}, actual ${REFERENCE_CONTENT_SHA256}"
  )
endif()
set_property(
  DIRECTORY
  APPEND
  PROPERTY CMAKE_CONFIGURE_DEPENDS
           ${REFERENCE_ADAPTER_DEPENDS}
)
if(CMAKE_PRESET_NAME)
  set(REFERENCE_PRESET "${CMAKE_PRESET_NAME}")
else()
  cmake_path(GET CMAKE_BINARY_DIR FILENAME REFERENCE_PRESET)
endif()
string(TOUPPER "${CMAKE_BUILD_TYPE}" REFERENCE_BUILD_TYPE_UPPER)
set(REFERENCE_CONFIG_COMPILE_FLAGS_VARIABLE "CMAKE_CXX_FLAGS_${REFERENCE_BUILD_TYPE_UPPER}")
set(REFERENCE_CONFIG_LINK_FLAGS_VARIABLE "CMAKE_EXE_LINKER_FLAGS_${REFERENCE_BUILD_TYPE_UPPER}")
string(
  STRIP
  "${CMAKE_CXX_FLAGS} ${${REFERENCE_CONFIG_COMPILE_FLAGS_VARIABLE}}"
  REFERENCE_COMPILE_FLAGS
)
string(
  STRIP
  "${CMAKE_EXE_LINKER_FLAGS} ${${REFERENCE_CONFIG_LINK_FLAGS_VARIABLE}}"
  REFERENCE_LINK_FLAGS
)
if(NOT REFERENCE_COMPILE_FLAGS)
  set(REFERENCE_COMPILE_FLAGS "<none>")
endif()
if(NOT REFERENCE_LINK_FLAGS)
  set(REFERENCE_LINK_FLAGS "<none>")
endif()
if(REFERENCE_COMPILE_FLAGS MATCHES "sanitize")
  set(REFERENCE_SANITIZER_MODE "asan-ubsan")
else()
  set(REFERENCE_SANITIZER_MODE "none")
endif()
if(CMAKE_CXX_COMPILER_TARGET)
  set(REFERENCE_COMPILER_TARGET "${CMAKE_CXX_COMPILER_TARGET}")
else()
  execute_process(
    COMMAND "${CMAKE_CXX_COMPILER}" -dumpmachine
    OUTPUT_VARIABLE REFERENCE_COMPILER_TARGET
    OUTPUT_STRIP_TRAILING_WHITESPACE
    ERROR_QUIET
  )
endif()
if(NOT REFERENCE_COMPILER_TARGET)
  set(REFERENCE_COMPILER_TARGET "${CMAKE_SYSTEM_PROCESSOR}-${CMAKE_SYSTEM_NAME}")
endif()
set(
  REFERENCE_TARGET
  "${REFERENCE_COMPILER_TARGET}|${CMAKE_SYSTEM_NAME}|${CMAKE_SYSTEM_PROCESSOR}"
)
set(REFERENCE_TARGET_CPU "baseline")
if(REFERENCE_USER_COMPILE_FLAGS MATCHES "(-march=native|-mcpu=native)")
  set(REFERENCE_TARGET_CPU "native")
endif()
set(REFERENCE_TARGET_FEATURES "<none>")
if(CMAKE_SYSROOT)
  set(REFERENCE_SDK_OR_SYSROOT "${CMAKE_SYSROOT}")
elseif(CMAKE_OSX_SYSROOT)
  set(REFERENCE_SDK_OR_SYSROOT "${CMAKE_OSX_SYSROOT}")
else()
  set(REFERENCE_SDK_OR_SYSROOT "<none>")
endif()
if(CMAKE_BUILD_TYPE STREQUAL "Debug")
  set(REFERENCE_OPTIMIZATION "O0")
else()
  set(REFERENCE_OPTIMIZATION "O3")
endif()
list(FIND REFERENCE_SUPPORTED_FP_OPTIONS "-ffp-contract=off" REFERENCE_FP_CONTRACT_INDEX)
if(REFERENCE_FP_CONTRACT_INDEX EQUAL -1)
  set(REFERENCE_FP_CONTRACT "unsupported-d2")
else()
  set(REFERENCE_FP_CONTRACT "off")
endif()
list(FIND REFERENCE_SUPPORTED_FP_OPTIONS "-fdenormal-fp-math=ieee" REFERENCE_DENORMAL_INDEX)
list(FIND REFERENCE_SUPPORTED_FP_OPTIONS "-fdenormal-fp-math-fp32=ieee" REFERENCE_DENORMAL_FP32_INDEX)
if(REFERENCE_DENORMAL_INDEX EQUAL -1 OR REFERENCE_DENORMAL_FP32_INDEX EQUAL -1)
  set(REFERENCE_DENORMAL_MODE "ieee-requested-d2")
else()
  set(REFERENCE_DENORMAL_MODE "ieee")
endif()
if(REFERENCE_MISSING_FP_OPTIONS)
  string(JOIN "," REFERENCE_MISSING_FP_FLAGS ${REFERENCE_MISSING_FP_OPTIONS})
  set(REFERENCE_FP_MODEL "precise-supported-d2")
else()
  set(REFERENCE_MISSING_FP_FLAGS "<none>")
  set(REFERENCE_FP_MODEL "precise")
endif()
if(CMAKE_SYSTEM_NAME STREQUAL "Linux")
  set(REFERENCE_OS "linux")
  set(REFERENCE_LIBC "glibc")
  set(REFERENCE_LIBM "libm")
elseif(CMAKE_SYSTEM_NAME STREQUAL "Darwin")
  set(REFERENCE_OS "macos")
  set(REFERENCE_LIBC "libSystem")
  set(REFERENCE_LIBM "libSystem")
elseif(CMAKE_SYSTEM_NAME STREQUAL "Windows")
  set(REFERENCE_OS "windows")
  set(REFERENCE_LIBC "msvcrt")
  set(REFERENCE_LIBM "msvcrt")
else()
  set(REFERENCE_OS "${CMAKE_SYSTEM_NAME}")
  set(REFERENCE_LIBC "<unavailable-d3>")
  set(REFERENCE_LIBM "<unavailable-d3>")
endif()

if(MSVC)
  set(REFERENCE_WARNING_OPTIONS /W4 /WX)
else()
  set(REFERENCE_WARNING_OPTIONS -Wall -Wextra -Wpedantic -Werror)
  if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    list(APPEND REFERENCE_WARNING_OPTIONS -Wno-overriding-option)
  endif()
endif()
string(JOIN " " REFERENCE_WARNING_FLAGS ${REFERENCE_WARNING_OPTIONS})
string(JOIN " " REFERENCE_FP_FLAGS ${REFERENCE_SUPPORTED_FP_OPTIONS})
string(
  JOIN
  " "
  REFERENCE_UPSTREAM_COMPATIBILITY_FLAGS
  ${REFERENCE_UPSTREAM_COMPATIBILITY_OPTIONS}
)
string(
  STRIP
  "${REFERENCE_COMPILE_FLAGS} ${REFERENCE_WARNING_FLAGS} ${REFERENCE_FP_FLAGS} ${REFERENCE_UPSTREAM_COMPATIBILITY_FLAGS} cxx_std_17"
  REFERENCE_COMPILE_FLAGS
)
set(
  REFERENCE_FEATURE_SET
  "evidence=${REFERENCE_EVIDENCE_TIER};applied=${REFERENCE_FP_FLAGS};missing=${REFERENCE_MISSING_FP_FLAGS}"
)
set(
  REFERENCE_COMPILE_COMMAND_DESCRIPTOR
  "compiler=${CMAKE_CXX_COMPILER};id=${CMAKE_CXX_COMPILER_ID};version=${CMAKE_CXX_COMPILER_VERSION};target=${REFERENCE_COMPILER_TARGET};build=${CMAKE_BUILD_TYPE};flags=${REFERENCE_COMPILE_FLAGS};sources=math_probe.cpp,protocol_bits.cpp"
)
string(SHA256 REFERENCE_COMPILE_COMMAND_SHA256 "${REFERENCE_COMPILE_COMMAND_DESCRIPTOR}")

function(reference_escape_cpp_string INPUT_VALUE OUTPUT_VARIABLE)
  string(REPLACE "\\" "\\\\" ESCAPED_VALUE "${INPUT_VALUE}")
  string(REPLACE "\"" "\\\"" ESCAPED_VALUE "${ESCAPED_VALUE}")
  string(REPLACE "\n" "\\n" ESCAPED_VALUE "${ESCAPED_VALUE}")
  string(REPLACE "\r" "\\r" ESCAPED_VALUE "${ESCAPED_VALUE}")
  set("${OUTPUT_VARIABLE}" "${ESCAPED_VALUE}" PARENT_SCOPE)
endfunction()

set(REFERENCE_ADAPTER_REVISION "source-digest-v1")
reference_escape_cpp_string(
  "${LIQUIDFUN_EXPECTED_ORACLE_REVISION}"
  REFERENCE_ORACLE_REVISION_CPP
)
reference_escape_cpp_string(
  "${REFERENCE_ADAPTER_REVISION}"
  REFERENCE_ADAPTER_REVISION_CPP
)
reference_escape_cpp_string(
  "${REFERENCE_CONTENT_SHA256}"
  REFERENCE_ADAPTER_CONTENT_SHA256_CPP
)
reference_escape_cpp_string("${REFERENCE_PRESET}" REFERENCE_PRESET_CPP)
reference_escape_cpp_string("${CMAKE_CXX_COMPILER_ID}" REFERENCE_COMPILER_ID_CPP)
reference_escape_cpp_string(
  "${CMAKE_CXX_COMPILER_VERSION}"
  REFERENCE_COMPILER_VERSION_CPP
)
reference_escape_cpp_string("${REFERENCE_TARGET}" REFERENCE_TARGET_CPP)
reference_escape_cpp_string("${CMAKE_BUILD_TYPE}" REFERENCE_BUILD_TYPE_CPP)
reference_escape_cpp_string(
  "${REFERENCE_COMPILE_FLAGS}"
  REFERENCE_COMPILE_FLAGS_CPP
)
reference_escape_cpp_string("${REFERENCE_LINK_FLAGS}" REFERENCE_LINK_FLAGS_CPP)
reference_escape_cpp_string(
  "${REFERENCE_SANITIZER_MODE}"
  REFERENCE_SANITIZER_MODE_CPP
)
reference_escape_cpp_string(
  "${REFERENCE_COMPILE_COMMAND_SHA256}"
  REFERENCE_COMPILE_COMMAND_SHA256_CPP
)
reference_escape_cpp_string("${REFERENCE_COMPILER_TARGET}" REFERENCE_TARGET_TRIPLE_CPP)
reference_escape_cpp_string("${REFERENCE_TARGET_CPU}" REFERENCE_TARGET_CPU_CPP)
reference_escape_cpp_string("${REFERENCE_TARGET_FEATURES}" REFERENCE_TARGET_FEATURES_CPP)
reference_escape_cpp_string("${REFERENCE_SDK_OR_SYSROOT}" REFERENCE_SDK_OR_SYSROOT_CPP)
reference_escape_cpp_string("${REFERENCE_OPTIMIZATION}" REFERENCE_OPTIMIZATION_CPP)
reference_escape_cpp_string("${REFERENCE_FP_MODEL}" REFERENCE_FP_MODEL_CPP)
reference_escape_cpp_string("${REFERENCE_FP_CONTRACT}" REFERENCE_FP_CONTRACT_CPP)
reference_escape_cpp_string("${REFERENCE_DENORMAL_MODE}" REFERENCE_DENORMAL_MODE_CPP)
reference_escape_cpp_string("${REFERENCE_FEATURE_SET}" REFERENCE_FEATURE_SET_CPP)
reference_escape_cpp_string("${REFERENCE_OS}" REFERENCE_OS_CPP)
reference_escape_cpp_string("${REFERENCE_LIBC}" REFERENCE_LIBC_CPP)
reference_escape_cpp_string("${REFERENCE_LIBM}" REFERENCE_LIBM_CPP)

set(
  REFERENCE_BUILD_IDENTITY_HEADER
  "${CMAKE_CURRENT_BINARY_DIR}/generated/build_identity.hpp"
)
configure_file(
  "${CMAKE_CURRENT_LIST_DIR}/src/build_identity.hpp.in"
  "${REFERENCE_BUILD_IDENTITY_HEADER}"
  @ONLY
  NEWLINE_STYLE UNIX
)

add_executable(
  liquidfun-reference
  src/main.cpp
  "${REFERENCE_BUILD_IDENTITY_HEADER}"
)
target_compile_features(liquidfun-reference PRIVATE cxx_std_17)
target_include_directories(
  liquidfun-reference
  PRIVATE "${CMAKE_CURRENT_BINARY_DIR}/generated"
)
target_link_libraries(
  liquidfun-reference
  PRIVATE liquidfun-reference-protocol Box2D
)

add_executable(
  liquidfun-reference-protocol-tests
  tests/protocol_tests.cpp
)
target_compile_features(liquidfun-reference-protocol-tests PRIVATE cxx_std_17)
target_compile_definitions(
  liquidfun-reference-protocol-tests
  PRIVATE LIQUIDFUN_REPOSITORY_ROOT="${CMAKE_CURRENT_LIST_DIR}/../.."
)
target_link_libraries(
  liquidfun-reference-protocol-tests
  PRIVATE liquidfun-reference-protocol
)

target_compile_options(
  liquidfun-reference-protocol
  PRIVATE ${REFERENCE_WARNING_OPTIONS} ${REFERENCE_SUPPORTED_FP_OPTIONS}
)
target_compile_options(
  liquidfun-reference
  PRIVATE ${REFERENCE_WARNING_OPTIONS} ${REFERENCE_SUPPORTED_FP_OPTIONS}
)
target_compile_options(
  liquidfun-reference-protocol-tests
  PRIVATE ${REFERENCE_WARNING_OPTIONS} ${REFERENCE_SUPPORTED_FP_OPTIONS}
)

add_test(
  NAME liquidfun-reference-protocol
  COMMAND liquidfun-reference-protocol-tests
)
