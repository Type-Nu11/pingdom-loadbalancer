#include <iostream>
#include <thread>
#include <chrono>

#include "monitor/HealthMonitor.hpp"
#include "haproxy/HAProxyClient.hpp"


HealthMonitor::HealthMonitor(
    HAProxyClient& client
)
    : haproxyClient(client),
      running(false)
{

}



void HealthMonitor::start()
{
    running = true;


    while(running)
    {

        std::string stats =
            haproxyClient.getStats();


        std::cout
            << "===== HAProxy Status ====="
            << std::endl;


        std::cout
            << stats
            << std::endl;



        std::this_thread::sleep_for(
            std::chrono::seconds(5)
        );

    }

}



void HealthMonitor::stop()
{
    running = false;
}