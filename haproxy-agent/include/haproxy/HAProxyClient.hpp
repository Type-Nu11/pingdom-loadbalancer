#ifndef HAPROXY_CLIENT_HPP
#define HAPROXY_CLIENT_HPP

#include <string>


class HAProxyClient
{

private:

    int socketFd;

    std::string socketPath;


public:

    HAProxyClient(
        const std::string& path = "/tmp/haproxy.sock"
    );


    ~HAProxyClient();


    bool connect();


    std::string execute(
        const std::string& command
    );


    std::string getInfo();


    std::string getStats();

};


#endif