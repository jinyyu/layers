cmake_minimum_required(VERSION 3.5)
project(layers-distribution)

install(DIRECTORY ${CMAKE_SOURCE_DIR}/etc DESTINATION /)


set(SRC
    cpp/checksum.cpp
    cpp/detector.cpp
    )

add_library(layerscpp ${SRC})

install(TARGETS layerscpp
        ARCHIVE DESTINATION lib)

find_package(PkgConfig REQUIRED)
pkg_check_modules(NDPI libndpi>=2.4.0 REQUIRED)
include_directories(${GNDPI_INCLUDE_DIRS})
set(LIBS ${LIBS} ${NDPI_LIBRARIES})

add_executable(main cpp/main.cpp)


