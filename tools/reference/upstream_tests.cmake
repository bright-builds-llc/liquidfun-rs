# Original build wrapper for the pinned, unmodified LiquidFun test sources.
# Sources retain their upstream notices: LiquidFun/Box2D zlib and GoogleTest BSD;
# see THIRD_PARTY_NOTICES.md. Avoid upstream Unittests/CMakeLists.txt because it
# forces executable outputs into the read-only upstream checkout.
set(UPSTREAM_ROOT "${CMAKE_CURRENT_LIST_DIR}/../../third_party/liquidfun")
set(UPSTREAM_TEST_ROOT "${UPSTREAM_ROOT}/liquidfun/Box2D/Unittests")
set(UPSTREAM_GTEST_ROOT "${UPSTREAM_ROOT}/googletest")
find_package(Threads REQUIRED)

add_library(liquidfun-upstream-gtest STATIC "${UPSTREAM_GTEST_ROOT}/src/gtest-all.cc")
target_include_directories(liquidfun-upstream-gtest SYSTEM PUBLIC "${UPSTREAM_GTEST_ROOT}/include")
target_include_directories(liquidfun-upstream-gtest PRIVATE "${UPSTREAM_GTEST_ROOT}")
target_link_libraries(liquidfun-upstream-gtest PUBLIC Threads::Threads)
set_target_properties(liquidfun-upstream-gtest PROPERTIES CXX_STANDARD 11 CXX_STANDARD_REQUIRED ON)
target_compile_options(liquidfun-upstream-gtest PRIVATE -Wall -Werror)
if(CMAKE_CXX_COMPILER_ID MATCHES "Clang" AND CMAKE_CXX_COMPILER_VERSION VERSION_GREATER_EQUAL "22")
  # Pinned gtest's StackGrowsDown passes an uninitialized local's address only
  # for stack-address comparison; StackLowerThanAddress never reads its value.
  # Keep Clang 22's warning visible on this third-party translation unit alone.
  target_compile_options(liquidfun-upstream-gtest PRIVATE -Wno-error=uninitialized-const-pointer)
endif()

set(UPSTREAM_TEST_NAMES
  BlockAllocator BodyContacts Callback Color Common Confinement Conservation
  FreeList Function HelloWorld IntrusiveList SlabAllocator TrackedBlock
)
add_custom_target(liquidfun-upstream-tests)
# The baseline and generated comparison output live next to the test binaries.
file(COPY "${UPSTREAM_TEST_ROOT}/baselines" DESTINATION "${CMAKE_BINARY_DIR}")
file(MAKE_DIRECTORY "${CMAKE_BINARY_DIR}/output")
foreach(test_name IN LISTS UPSTREAM_TEST_NAMES)
  set(test_target "upstream-${test_name}")
  add_executable(${test_target}
    "${UPSTREAM_TEST_ROOT}/${test_name}/${test_name}Tests.cpp"
    "${UPSTREAM_TEST_ROOT}/BodyTracker.cpp"
  )
  target_include_directories(${test_target} SYSTEM PRIVATE
    "${UPSTREAM_TEST_ROOT}" "${UPSTREAM_ROOT}/liquidfun/Box2D"
  )
  target_compile_definitions(${test_target} PRIVATE LIQUIDFUN_UNIT_TESTS=1)
  target_compile_options(${test_target} PRIVATE -Wall -Werror ${REFERENCE_SUPPORTED_FP_OPTIONS})
  target_link_libraries(${test_target} PRIVATE liquidfun-upstream-gtest Box2D)
  set_target_properties(${test_target} PROPERTIES
    CXX_STANDARD 11 CXX_STANDARD_REQUIRED ON
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}"
  )
  add_dependencies(liquidfun-upstream-tests ${test_target})
  add_test(NAME ${test_target} COMMAND ${test_target})
  set_tests_properties(${test_target} PROPERTIES TIMEOUT 120)
endforeach()
