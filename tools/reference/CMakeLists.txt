cmake_minimum_required(VERSION 3.25)

project(liquidfun_reference LANGUAGES C CXX)

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

add_subdirectory(
  "${CMAKE_CURRENT_LIST_DIR}/../../third_party/liquidfun/liquidfun/Box2D"
  "${CMAKE_BINARY_DIR}/upstream"
)

set(REFERENCE_UPSTREAM_COMPATIBILITY_OPTIONS "")
if(
  CMAKE_CXX_COMPILER_ID MATCHES "Clang"
  AND CMAKE_CXX_FLAGS MATCHES "sanitize"
)
  # The read-only 2014 oracle trips two modern Clang warnings under its own
  # -Werror policy. Keep warnings visible without mutating upstream sources.
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
  set(REFERENCE_COMPILER_TARGET "native")
endif()
set(
  REFERENCE_TARGET
  "${REFERENCE_COMPILER_TARGET}|${CMAKE_SYSTEM_NAME}|${CMAKE_SYSTEM_PROCESSOR}"
)

if(MSVC)
  set(REFERENCE_WARNING_OPTIONS /W4 /WX)
else()
  set(REFERENCE_WARNING_OPTIONS -Wall -Wextra -Wpedantic -Werror)
endif()
string(JOIN " " REFERENCE_WARNING_FLAGS ${REFERENCE_WARNING_OPTIONS})
string(
  JOIN
  " "
  REFERENCE_UPSTREAM_COMPATIBILITY_FLAGS
  ${REFERENCE_UPSTREAM_COMPATIBILITY_OPTIONS}
)
string(
  STRIP
  "${REFERENCE_COMPILE_FLAGS} ${REFERENCE_WARNING_FLAGS} ${REFERENCE_UPSTREAM_COMPATIBILITY_FLAGS} cxx_std_17"
  REFERENCE_COMPILE_FLAGS
)

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
  PRIVATE ${REFERENCE_WARNING_OPTIONS}
)
target_compile_options(liquidfun-reference PRIVATE ${REFERENCE_WARNING_OPTIONS})
target_compile_options(
  liquidfun-reference-protocol-tests
  PRIVATE ${REFERENCE_WARNING_OPTIONS}
)

add_test(
  NAME liquidfun-reference-protocol
  COMMAND liquidfun-reference-protocol-tests
)
