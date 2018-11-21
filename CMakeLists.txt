cmake_minimum_required(VERSION 3.5)
project(layers-distribution)

install(DIRECTORY ${CMAKE_SOURCE_DIR}/etc DESTINATION /)


set(SRC
    cpp/checksum.cpp
    )

add_library(layerscpp ${SRC})

install(TARGETS layerscpp
        ARCHIVE DESTINATION lib)


add_executable(main cpp/main.cpp)


