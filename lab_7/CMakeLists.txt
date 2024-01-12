cmake_minimum_required(VERSION 3.25)
project(lab_7)

set(CMAKE_CXX_STANDARD 17)

find_package(MPI REQUIRED)

add_executable(lab_7 main.cpp)
target_link_libraries(lab_7 PRIVATE MPI::MPI_CXX)
