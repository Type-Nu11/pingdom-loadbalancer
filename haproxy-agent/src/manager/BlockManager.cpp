#ifndef BLOCK_MANAGER_HPP
#define BLOCK_MANAGER_HPP

#include <string>
#include <vector>

#include "model/BlockEntry.hpp"


class BlockManager
{
private:

    std::vector<BlockEntry> blockedIps;


public:

    void block(
        const std::string& ipAddress,
        const std::string& reason
    );


    void unblock(
        const std::string& ipAddress
    );


    bool isBlocked(
        const std::string& ipAddress
    ) const;


    const std::vector<BlockEntry>& getBlockedIps() const;

};


#endif