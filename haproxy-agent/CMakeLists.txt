cmake_minimum_required(VERSION 3.20)

project(haproxy-agent)


set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)


include_directories(
    include
)


add_executable(
    haproxy-agent

    src/main.cpp

    src/haproxy/HAProxyClient.cpp

    src/monitor/HealthMonitor.cpp
)