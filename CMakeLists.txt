cmake_minimum_required(VERSION 3.5)
project(layers-distribution)

install(DIRECTORY ${CMAKE_SOURCE_DIR}/etc DESTINATION /)

set(CMAKE_CXX_STANDARD 11)

set(SRC
    cpp/checksum.cpp
    cpp/detector.cpp
    )

add_library(layerscpp SHARED ${SRC})

install(TARGETS layerscpp LIBRARY DESTINATION lib)

find_package(PkgConfig REQUIRED)
pkg_check_modules(NDPI libndpi>=2.4.0 REQUIRED)
include_directories(${NDPI_INCLUDE_DIRS})
set(LIBS ${LIBS} ${NDPI_LIBRARIES})

add_executable(main cpp/main.cpp)


