#include <iostream>

#include "haproxy/HaproxyClient.hpp"

int main() {
    std::cout << "- HAProxy Agent -" << std::endl;

    HAProxyClient client;

    if (!client.connect()) {
        std::cerr
            << "Failed to connect to HAProxy socket." 
            << std::endl;

        return 1;
    }

    std::cout
        << "[INFO] Connected"
        << std::endl;

    std::string info = client.getInfo();

    std::cout
        << "\n===== HAProxy Info =====\n" 
        << info
        << std::endl;


    return 0;
}