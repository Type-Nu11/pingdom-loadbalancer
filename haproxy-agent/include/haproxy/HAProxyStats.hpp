#ifndef HAPROXY_STATS_HPP
#define HAPROXY_STATS_HPP

#include <string>


class HAProxyStats {

    public:
        std::string proxyName;
        std::string serverName;
        std::string address;
        std::string status;
};

#endif