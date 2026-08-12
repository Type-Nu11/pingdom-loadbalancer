#ifndef BLOCKENTRY_HPP
#define BLOCKENTRY_HPP

#include <string>

struct BlockEntry {
    public:
        std::string ipAddress;
        std::string reason;
        std::string timestamp;
};

#endif