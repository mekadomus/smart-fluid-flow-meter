cmake_minimum_required(VERSION 3.27.4)
project(smart-fluid-flow-meter-test
  VERSION 0.1.0
  LANGUAGES CXX)

# Needs to be before ../include so it overwrites headers from main
include_directories(mock/test-mocks/include)
include_directories(../include)
include_directories(external/catch2)
include_directories(mock/esp-idf-mocks)

FILE(GLOB MOCK_SRCS mock/esp-idf-mocks/*.cpp
                    mock/esp-idf-mocks/lwip/*.cpp
                    mock/esp-idf-mocks/freertos/*.cpp)

set(TESTING_SRCS
  ../src/backend-service.cpp
)

set(TEST_TARGET_SRCS
  src/backend-service-test.cpp
)

add_compile_options(-Wall -Wextra -Wpedantic -Werror -std=c++20)

add_executable(
  test
  ${MOCK_SRCS}
  ${TESTING_SRCS}
  ${TEST_TARGET_SRCS}
)
