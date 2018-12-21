cmake_minimum_required(VERSION 3.5)
project(layers)

install(DIRECTORY ${CMAKE_SOURCE_DIR}/etc DESTINATION /)

set(CMAKE_CXX_STANDARD 11)

add_definitions(-Wall)

set(SRC
    cpp/checksum.cpp
    cpp/detector.cpp
    cpp/TCPDataTracker.cpp
    cpp/http_parser.c
    )

add_library(layerscpp SHARED ${SRC})

install(TARGETS layerscpp LIBRARY DESTINATION lib)

find_package(PkgConfig REQUIRED)
pkg_check_modules(NDPI libndpi>=2.4.0 REQUIRED)
include_directories(${NDPI_INCLUDE_DIRS})
set(LIBS ${LIBS} libndpi.so)

add_executable(main cpp/main.cpp)
target_link_libraries(main layerscpp ${LIBS})