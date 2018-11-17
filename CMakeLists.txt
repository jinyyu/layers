cmake_minimum_required(VERSION 3.5)
project(layers-distribution)

install(DIRECTORY ${CMAKE_SOURCE_DIR}/etc DESTINATION /)


add_executable(cpp cpp/main.cpp)