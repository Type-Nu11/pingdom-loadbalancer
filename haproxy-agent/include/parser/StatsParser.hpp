#ifndef HAPROXY_PARSER_HPP
#define HAPROXY_PARSER_HPP

#include <string>
#include <vector>

#include "haproxy/HAProxyStats.hpp"

class StatsParser {

    public:
        static std::vector<HAProxyStats> parse(
            const std::string& raw
        );
};

#endif