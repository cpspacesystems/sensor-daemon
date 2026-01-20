cmake_minimum_required(VERSION 3.16)
project(bmp390_test C)

file(GLOB SRC_FILES "src/*.c")
add_executable(bmp390_test ${SRC_FILES})
