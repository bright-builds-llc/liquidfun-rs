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
  -ffp-contract=off
  -fsigned-zeros
  -fhonor-nans
  -fhonor-infinities
  -fdenormal-fp-math=ieee
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
  "(^|[ ;])(-ffast-math|-Ofast|-fassociative-math|-freciprocal-math|-funsafe-math-optimizations|-march=[^ ;]+|-mcpu=[^ ;]+|-mtune=[^ ;]+|-mavx[^ ;]*|-mfma|-msse[^ ;]*|-mllvm|-Xclang)($|[ ;])|unsafe-fp|no-nans-fp|no-infs-fp|fp-contract=(fast|on)"
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

if(CMAKE_PRESET_NAME)
  set(REFERENCE_PRESET "${CMAKE_PRESET_NAME}")
else()
  cmake_path(GET CMAKE_BINARY_DIR FILENAME REFERENCE_PRESET)
endif()

add_subdirectory(
  "${CMAKE_CURRENT_LIST_DIR}/../../third_party/liquidfun/liquidfun/Box2D"
  "${CMAKE_BINARY_DIR}/upstream"
)
target_compile_options(Box2D PRIVATE ${REFERENCE_SUPPORTED_FP_OPTIONS})

file(
  REAL_PATH
  "${CMAKE_CURRENT_LIST_DIR}/../../third_party/liquidfun/liquidfun/Box2D/Box2D/Particle/b2ParticleSystem.cpp"
  REFERENCE_PARTICLE_SYSTEM_SOURCE
)
set(REFERENCE_EXPECT_PARTICLE_SHIFT_BASE_EXCEPTION FALSE)
if(REFERENCE_PRESET STREQUAL "oracle-asan-ubsan")
  # Pinned upstream computeRelativeTag intentionally shifts the x=-1 neighbor
  # offset. Keep the approved legacy exception on that translation unit only;
  # every repository-owned source and every other sanitizer category remains
  # fully instrumented.
  set_property(
    SOURCE "${REFERENCE_PARTICLE_SYSTEM_SOURCE}"
    TARGET_DIRECTORY Box2D
    APPEND
    PROPERTY COMPILE_OPTIONS -fno-sanitize=shift-base
  )
  set(REFERENCE_EXPECT_PARTICLE_SHIFT_BASE_EXCEPTION TRUE)
endif()

set(REFERENCE_UPSTREAM_COMPATIBILITY_OPTIONS "")
if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
  # The read-only 2014 oracle trips modern Clang warnings under its own
  # -Werror policy. Keep warnings visible without mutating upstream sources or
  # weakening repository-authored code.
  set(
    REFERENCE_UPSTREAM_COMPATIBILITY_OPTIONS
    -Wno-error=deprecated-copy-with-user-provided-copy
    -Wno-error=unused-result
    -Wno-error=nontrivial-memcall
  )
  if(CMAKE_BUILD_TYPE STREQUAL "Release")
    list(
      APPEND
      REFERENCE_UPSTREAM_COMPATIBILITY_OPTIONS
      -Wno-error=unused-but-set-variable
    )
  endif()
  if(REFERENCE_USER_COMPILE_FLAGS MATCHES "(^|[ ;])-fsanitize=")
    list(
      APPEND
      REFERENCE_UPSTREAM_COMPATIBILITY_OPTIONS
      -Wno-error=overriding-option
    )
  endif()
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
  liquidfun-reference-protocol-bits
  STATIC
  src/protocol_bits.cpp
)
target_compile_features(liquidfun-reference-protocol-bits PUBLIC cxx_std_17)
target_include_directories(
  liquidfun-reference-protocol-bits
  PUBLIC "${CMAKE_CURRENT_LIST_DIR}/src"
  PRIVATE "${CMAKE_CURRENT_LIST_DIR}/vendor"
)
target_include_directories(
  liquidfun-reference-protocol-bits
  SYSTEM PRIVATE
  "${CMAKE_CURRENT_LIST_DIR}/../../third_party/liquidfun/liquidfun/Box2D"
)

add_library(
  liquidfun-reference-protocol
  STATIC
  src/benchmark_run.cpp
  src/catalog_checkpoint.cpp
  src/catalog_joint.cpp
  src/catalog_run.cpp
  src/catalog_run_decode.cpp
  src/catalog_run_session.cpp
  src/collision_probe.cpp
  src/math_probe.cpp
  src/protocol.cpp
  src/rigid_world.cpp
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
target_link_libraries(
  liquidfun-reference-protocol
  PRIVATE liquidfun-reference-protocol-bits Box2D
)

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
set(REFERENCE_TARGET "${REFERENCE_COMPILER_TARGET}")
set(REFERENCE_TARGET_CPU "baseline")
if(REFERENCE_USER_COMPILE_FLAGS MATCHES "(-march=|-mcpu=|-mtune=)")
  set(REFERENCE_TARGET_CPU "explicit")
endif()
set(REFERENCE_TARGET_FEATURES "<none>")
if(REFERENCE_USER_COMPILE_FLAGS MATCHES "(-mavx|-mfma|-msse)")
  set(REFERENCE_TARGET_FEATURES "explicit")
endif()
if(CMAKE_SYSROOT)
  set(REFERENCE_SDK_OR_SYSROOT "<configured>")
elseif(CMAKE_OSX_SYSROOT)
  set(REFERENCE_SDK_OR_SYSROOT "<configured>")
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
if(REFERENCE_DENORMAL_INDEX EQUAL -1)
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
  execute_process(
    COMMAND getconf GNU_LIBC_VERSION
    OUTPUT_VARIABLE REFERENCE_GLIBC_VERSION
    OUTPUT_STRIP_TRAILING_WHITESPACE
    ERROR_QUIET
  )
  if(NOT REFERENCE_GLIBC_VERSION)
    set(REFERENCE_GLIBC_VERSION "glibc@${CMAKE_SYSTEM_VERSION}")
  endif()
  set(REFERENCE_LIBC "${REFERENCE_GLIBC_VERSION}")
  set(REFERENCE_LIBM "libm@${REFERENCE_GLIBC_VERSION}")
elseif(CMAKE_SYSTEM_NAME STREQUAL "Darwin")
  set(REFERENCE_OS "macos")
  set(REFERENCE_LIBC "libSystem@Darwin-${CMAKE_SYSTEM_VERSION}")
  set(REFERENCE_LIBM "libSystem-libm@Darwin-${CMAKE_SYSTEM_VERSION}")
elseif(CMAKE_SYSTEM_NAME STREQUAL "Windows")
  set(REFERENCE_OS "windows")
  set(REFERENCE_LIBC "ucrt@Windows-${CMAKE_SYSTEM_VERSION}")
  set(REFERENCE_LIBM "ucrt-libm@Windows-${CMAKE_SYSTEM_VERSION}")
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
  "evidence=${REFERENCE_EVIDENCE_TIER};applied=${REFERENCE_FP_FLAGS};missing=${REFERENCE_MISSING_FP_FLAGS};compile_database=collision_probe.cpp,math_probe.cpp,protocol_bits.cpp,rigid_world.cpp"
)
set(
  REFERENCE_PUBLIC_COMPILE_SUMMARY
  "optimization=${REFERENCE_OPTIMIZATION};fp_model=${REFERENCE_FP_MODEL};fp_contract=${REFERENCE_FP_CONTRACT};denormal=${REFERENCE_DENORMAL_MODE};target_cpu=${REFERENCE_TARGET_CPU};target_features=${REFERENCE_TARGET_FEATURES}"
)
set(REFERENCE_PUBLIC_LINK_SUMMARY "sanitizer=${REFERENCE_SANITIZER_MODE}")

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
  "${REFERENCE_PUBLIC_COMPILE_SUMMARY}"
  REFERENCE_PUBLIC_COMPILE_SUMMARY_CPP
)
reference_escape_cpp_string(
  "${REFERENCE_PUBLIC_LINK_SUMMARY}"
  REFERENCE_PUBLIC_LINK_SUMMARY_CPP
)
reference_escape_cpp_string(
  "${REFERENCE_SANITIZER_MODE}"
  REFERENCE_SANITIZER_MODE_CPP
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
file(REAL_PATH "${CMAKE_CURRENT_LIST_DIR}/../.." REFERENCE_REPOSITORY_ROOT)

set(
  REFERENCE_BUILD_IDENTITY_HEADER
  "${CMAKE_CURRENT_BINARY_DIR}/generated/build_identity.hpp"
)
set(
  REFERENCE_BUILD_IDENTITY_SCRIPT
  "${CMAKE_CURRENT_BINARY_DIR}/generated/generate_build_identity.cmake"
)
configure_file(
  "${CMAKE_CURRENT_LIST_DIR}/src/generate_build_identity.cmake.in"
  "${REFERENCE_BUILD_IDENTITY_SCRIPT}"
  @ONLY
  NEWLINE_STYLE UNIX
)
add_custom_command(
  OUTPUT "${REFERENCE_BUILD_IDENTITY_HEADER}"
  COMMAND "${CMAKE_COMMAND}" -P "${REFERENCE_BUILD_IDENTITY_SCRIPT}"
  DEPENDS
    "${CMAKE_BINARY_DIR}/compile_commands.json"
    "${REFERENCE_BUILD_IDENTITY_SCRIPT}"
    "${CMAKE_CURRENT_LIST_DIR}/src/benchmark_run.cpp"
    "${CMAKE_CURRENT_LIST_DIR}/src/collision_probe.cpp"
    "${CMAKE_CURRENT_LIST_DIR}/src/catalog_checkpoint.cpp"
    "${CMAKE_CURRENT_LIST_DIR}/src/catalog_joint.cpp"
    "${CMAKE_CURRENT_LIST_DIR}/src/catalog_run.cpp"
    "${CMAKE_CURRENT_LIST_DIR}/src/catalog_run_decode.cpp"
    "${CMAKE_CURRENT_LIST_DIR}/src/catalog_run_session.cpp"
    "${CMAKE_CURRENT_LIST_DIR}/src/math_probe.cpp"
    "${CMAKE_CURRENT_LIST_DIR}/src/protocol_bits.cpp"
    "${CMAKE_CURRENT_LIST_DIR}/src/rigid_world.cpp"
  COMMENT "Hashing effective math/probe compile commands"
  VERBATIM
)
set_source_files_properties(
  "${REFERENCE_BUILD_IDENTITY_HEADER}"
  PROPERTIES GENERATED TRUE
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

file(
  SHA256
  "${CMAKE_CURRENT_LIST_DIR}/src/phase9_lifecycle_contact_witness.cpp"
  PHASE9_PROBE_SOURCE_SHA256
)
add_executable(
  phase9-lifecycle-contact-witness
  src/phase9_lifecycle_contact_witness.cpp
)
target_compile_features(phase9-lifecycle-contact-witness PRIVATE cxx_std_17)
target_compile_definitions(
  phase9-lifecycle-contact-witness
  PRIVATE
    PHASE9_CMAKE_PRESET="${REFERENCE_PRESET}"
    PHASE9_COMPILER_ID="${CMAKE_CXX_COMPILER_ID}"
    PHASE9_COMPILER_VERSION="${CMAKE_CXX_COMPILER_VERSION}"
    PHASE9_ORACLE_REVISION="${LIQUIDFUN_EXPECTED_ORACLE_REVISION}"
    PHASE9_PROBE_SOURCE_SHA256="${PHASE9_PROBE_SOURCE_SHA256}"
    PHASE9_TARGET="${REFERENCE_TARGET}"
)
target_include_directories(
  phase9-lifecycle-contact-witness
  PRIVATE
    "${CMAKE_CURRENT_LIST_DIR}/src"
    "${CMAKE_CURRENT_LIST_DIR}/vendor"
)
target_include_directories(
  phase9-lifecycle-contact-witness
  SYSTEM PRIVATE
  "${CMAKE_CURRENT_LIST_DIR}/../../third_party/liquidfun/liquidfun/Box2D"
)
target_link_libraries(
  phase9-lifecycle-contact-witness
  PRIVATE liquidfun-reference-protocol-bits Box2D
)

file(
  SHA256
  "${CMAKE_CURRENT_LIST_DIR}/src/phase10_group_topology_witness.cpp"
  PHASE10_GROUP_TOPOLOGY_MAIN_SHA256
)
file(
  SHA256
  "${CMAKE_CURRENT_LIST_DIR}/src/phase10_group_topology_cases.cpp"
  PHASE10_GROUP_TOPOLOGY_CASES_SHA256
)
file(
  SHA256
  "${CMAKE_CURRENT_LIST_DIR}/src/phase10_group_topology_cases.hpp"
  PHASE10_GROUP_TOPOLOGY_HEADER_SHA256
)
string(
  SHA256
  PHASE10_GROUP_TOPOLOGY_PROBE_SOURCE_SHA256
  "phase10_group_topology_witness.cpp=${PHASE10_GROUP_TOPOLOGY_MAIN_SHA256}\nphase10_group_topology_cases.cpp=${PHASE10_GROUP_TOPOLOGY_CASES_SHA256}\nphase10_group_topology_cases.hpp=${PHASE10_GROUP_TOPOLOGY_HEADER_SHA256}\n"
)
add_executable(
  phase10-group-topology-witness
  src/phase10_group_topology_witness.cpp
  src/phase10_group_topology_cases.cpp
  "${REFERENCE_BUILD_IDENTITY_HEADER}"
)
target_compile_features(phase10-group-topology-witness PRIVATE cxx_std_17)
target_compile_definitions(
  phase10-group-topology-witness
  PRIVATE
    PHASE10_GROUP_TOPOLOGY_PROBE_SOURCE_SHA256="${PHASE10_GROUP_TOPOLOGY_PROBE_SOURCE_SHA256}"
)
target_include_directories(
  phase10-group-topology-witness
  PRIVATE
    "${CMAKE_CURRENT_BINARY_DIR}/generated"
    "${CMAKE_CURRENT_LIST_DIR}/src"
    "${CMAKE_CURRENT_LIST_DIR}/vendor"
)
target_include_directories(
  phase10-group-topology-witness
  SYSTEM PRIVATE
  "${CMAKE_CURRENT_LIST_DIR}/../../third_party/liquidfun/liquidfun/Box2D"
)
target_link_libraries(
  phase10-group-topology-witness
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
  liquidfun-reference-protocol-bits
  PRIVATE ${REFERENCE_WARNING_OPTIONS} ${REFERENCE_SUPPORTED_FP_OPTIONS}
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
target_compile_options(
  phase9-lifecycle-contact-witness
  PRIVATE ${REFERENCE_WARNING_OPTIONS} ${REFERENCE_SUPPORTED_FP_OPTIONS}
)
target_compile_options(
  phase10-group-topology-witness
  PRIVATE ${REFERENCE_WARNING_OPTIONS} ${REFERENCE_SUPPORTED_FP_OPTIONS}
)

add_test(
  NAME liquidfun-reference-protocol
  COMMAND liquidfun-reference-protocol-tests
)
add_test(
  NAME liquidfun-reference-sanitizer-scope
  COMMAND
    "${CMAKE_COMMAND}"
    "-DCOMPILE_DATABASE=${CMAKE_BINARY_DIR}/compile_commands.json"
    "-DEXPECT_EXCEPTION=${REFERENCE_EXPECT_PARTICLE_SHIFT_BASE_EXCEPTION}"
    "-DEXPECTED_SOURCE=${REFERENCE_PARTICLE_SYSTEM_SOURCE}"
    -P
    "${CMAKE_CURRENT_LIST_DIR}/tests/sanitizer_scope.cmake"
)
