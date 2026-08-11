#ifndef BLOCKMANAGER_HPP
#define BLOCKMANAGER_HPP

#include <vector>

#include "haproxy/model/BlockEntry.hpp"

class BlockManager {

    private:
        std::vector<BlockEntry> blockedEntries;

    public:
        
        void add(
            const BlockEntry& entry
        );

        void remove(
            const std::string& ip
        );

        void contains(
            const std::string& ip
        );

        std::vector<BlockEntry> getAll();
};

#endif