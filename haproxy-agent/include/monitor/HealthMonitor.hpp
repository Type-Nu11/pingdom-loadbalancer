#ifndef HEALTHMONITOR_HPP
#define HEALTHMONITOR_HPP

#include "haproxy/HAProxyClient.hpp"

class HealthMonitor
{
private:
    HAProxyClient& haproxyClient;
    bool running;

public:
    HealthMonitor(
        HAProxyClient& client
    );

    void start();

    void stop();
};

#endif