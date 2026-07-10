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

include(CTest)

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

set_property(
  DIRECTORY
  APPEND
  PROPERTY CMAKE_CONFIGURE_DEPENDS
           "${CMAKE_CURRENT_LIST_DIR}/src/main.cpp"
           "${CMAKE_CURRENT_LIST_DIR}/src/protocol.cpp"
           "${CMAKE_CURRENT_LIST_DIR}/src/protocol_bits.cpp"
           "${CMAKE_CURRENT_LIST_DIR}/src/protocol.hpp"
           "${CMAKE_CURRENT_LIST_DIR}/src/oracle_adapter.cpp"
           "${CMAKE_CURRENT_LIST_DIR}/src/oracle_adapter.hpp"
)
file(SHA256 "${CMAKE_CURRENT_LIST_DIR}/src/main.cpp" REFERENCE_MAIN_SHA256)
file(SHA256 "${CMAKE_CURRENT_LIST_DIR}/src/protocol.cpp" REFERENCE_PROTOCOL_SHA256)
file(SHA256 "${CMAKE_CURRENT_LIST_DIR}/src/protocol_bits.cpp" REFERENCE_BITS_SHA256)
file(SHA256 "${CMAKE_CURRENT_LIST_DIR}/src/protocol.hpp" REFERENCE_PROTOCOL_HEADER_SHA256)
file(SHA256 "${CMAKE_CURRENT_LIST_DIR}/src/oracle_adapter.cpp" REFERENCE_ADAPTER_SHA256)
file(SHA256 "${CMAKE_CURRENT_LIST_DIR}/src/oracle_adapter.hpp" REFERENCE_ADAPTER_HEADER_SHA256)
string(
  SHA256
  REFERENCE_CONTENT_SHA256
  "${REFERENCE_MAIN_SHA256}:${REFERENCE_PROTOCOL_SHA256}:${REFERENCE_BITS_SHA256}:${REFERENCE_PROTOCOL_HEADER_SHA256}:${REFERENCE_ADAPTER_SHA256}:${REFERENCE_ADAPTER_HEADER_SHA256}"
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

add_executable(liquidfun-reference src/main.cpp)
target_compile_features(liquidfun-reference PRIVATE cxx_std_17)
target_compile_definitions(
  liquidfun-reference
  PRIVATE
    LIQUIDFUN_ADAPTER_CONTENT_SHA256="${REFERENCE_CONTENT_SHA256}"
    LIQUIDFUN_BUILD_TYPE="${CMAKE_BUILD_TYPE}"
    LIQUIDFUN_CMAKE_PRESET="${REFERENCE_PRESET}"
    LIQUIDFUN_COMPILER_ID="${CMAKE_CXX_COMPILER_ID}"
    LIQUIDFUN_COMPILER_VERSION="${CMAKE_CXX_COMPILER_VERSION}"
    LIQUIDFUN_EFFECTIVE_COMPILE_FLAGS="${REFERENCE_COMPILE_FLAGS}"
    LIQUIDFUN_EFFECTIVE_LINK_FLAGS="${REFERENCE_LINK_FLAGS}"
    LIQUIDFUN_SANITIZER_MODE="${REFERENCE_SANITIZER_MODE}"
    LIQUIDFUN_TARGET="${CMAKE_SYSTEM_NAME}-${CMAKE_SYSTEM_PROCESSOR}"
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

if(MSVC)
  target_compile_options(liquidfun-reference-protocol PRIVATE /W4 /WX)
  target_compile_options(liquidfun-reference PRIVATE /W4 /WX)
  target_compile_options(liquidfun-reference-protocol-tests PRIVATE /W4 /WX)
else()
  target_compile_options(
    liquidfun-reference-protocol
    PRIVATE -Wall -Wextra -Wpedantic -Werror
  )
  target_compile_options(liquidfun-reference PRIVATE -Wall -Wextra -Wpedantic -Werror)
  target_compile_options(
    liquidfun-reference-protocol-tests
    PRIVATE -Wall -Wextra -Wpedantic -Werror
  )
endif()

add_test(
  NAME liquidfun-reference-protocol
  COMMAND liquidfun-reference-protocol-tests
)
