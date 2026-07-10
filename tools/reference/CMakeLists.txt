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
