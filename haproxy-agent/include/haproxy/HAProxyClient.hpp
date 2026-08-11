#ifndef HAPROXY_CLIENT_HPP
#define HAPROXY_CLIENT_HPP

#include <string>

class HAProxyClient {

    private:
        int socketFd;

    public:
        HAProxyClient();

        ~HAProxyClient();

        bool connect();

        std::string execute(
            const std::string& command
        );

        std::string getInfo();
        std::string getStats();
};

#endif