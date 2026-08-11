#ifndef HAPROXY_STATS_HPP
#define HAPROXY_STATS_HPP

#include <string>


class HAProxuStats {

    public:
        std::string proxyName;
        std::string serverName;
        std::string status;
}

#endif