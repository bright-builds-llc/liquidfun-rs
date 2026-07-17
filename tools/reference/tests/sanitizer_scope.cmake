cmake_minimum_required(VERSION 3.25)

foreach(REQUIRED_VARIABLE IN ITEMS COMPILE_DATABASE EXPECT_EXCEPTION EXPECTED_SOURCE)
  if(NOT DEFINED ${REQUIRED_VARIABLE} OR "${${REQUIRED_VARIABLE}}" STREQUAL "")
    message(FATAL_ERROR "${REQUIRED_VARIABLE} is required")
  endif()
endforeach()
if(NOT EXISTS "${COMPILE_DATABASE}")
  message(FATAL_ERROR "compile database is unavailable: ${COMPILE_DATABASE}")
endif()

file(REAL_PATH "${EXPECTED_SOURCE}" NORMALIZED_EXPECTED_SOURCE)
file(READ "${COMPILE_DATABASE}" COMPILE_DATABASE_JSON)
string(JSON COMPILE_ENTRY_COUNT LENGTH "${COMPILE_DATABASE_JSON}")
set(EXCEPTION_COUNT 0)

if(COMPILE_ENTRY_COUNT GREATER 0)
  math(EXPR LAST_COMPILE_ENTRY "${COMPILE_ENTRY_COUNT} - 1")
  foreach(INDEX RANGE 0 ${LAST_COMPILE_ENTRY})
    string(JSON SOURCE_FILE GET "${COMPILE_DATABASE_JSON}" ${INDEX} file)
    string(
      JSON
      EFFECTIVE_COMMAND
      ERROR_VARIABLE COMMAND_ERROR
      GET "${COMPILE_DATABASE_JSON}" ${INDEX} command
    )
    if(COMMAND_ERROR)
      message(FATAL_ERROR "compile database entry ${INDEX} has no command")
    endif()
    if(NOT EFFECTIVE_COMMAND MATCHES "(^|[ ;])-fno-sanitize=shift-base($|[ ;])")
      continue()
    endif()

    math(EXPR EXCEPTION_COUNT "${EXCEPTION_COUNT} + 1")
    file(REAL_PATH "${SOURCE_FILE}" NORMALIZED_SOURCE_FILE)
    if(NOT NORMALIZED_SOURCE_FILE STREQUAL NORMALIZED_EXPECTED_SOURCE)
      message(
        FATAL_ERROR
        "shift-base exception leaked to ${NORMALIZED_SOURCE_FILE}"
      )
    endif()
    if(NOT EFFECTIVE_COMMAND MATCHES "(^|[ ;])-fsanitize=address,undefined($|[ ;])")
      message(FATAL_ERROR "particle source lost ASan/UBSan instrumentation")
    endif()
    if(NOT EFFECTIVE_COMMAND MATCHES "(^|[ ;])-fno-sanitize-recover=undefined($|[ ;])")
      message(FATAL_ERROR "particle source lost fail-fast UBSan behavior")
    endif()
    if(EFFECTIVE_COMMAND MATCHES "(^|[ ;])-fno-sanitize=(shift|shift-exponent)($|[ ;])")
      message(FATAL_ERROR "particle source disabled more than shift-base checks")
    endif()
  endforeach()
endif()

if(EXPECT_EXCEPTION)
  if(NOT EXCEPTION_COUNT EQUAL 1)
    message(
      FATAL_ERROR
      "sanitizer preset must have exactly one shift-base exception, found ${EXCEPTION_COUNT}"
    )
  endif()
elseif(NOT EXCEPTION_COUNT EQUAL 0)
  message(
    FATAL_ERROR
    "non-sanitizer preset contains ${EXCEPTION_COUNT} shift-base exceptions"
  )
endif()
