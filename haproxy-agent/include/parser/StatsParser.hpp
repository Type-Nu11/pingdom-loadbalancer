#ifndef HAPROXY_PARSER_HPP
#define HAPROXY_PARSER_HPP

#include <string>
#include <vector>

#include "haproxy/HaproxyStats.hpp"

class StatsParser {

    public:
        static std::vector<HAProxuStats> parse(
            const std::string& raw
        );
};