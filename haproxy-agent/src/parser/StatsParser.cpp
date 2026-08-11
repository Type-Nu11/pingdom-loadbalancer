#include "parser/StatsParser.hpp"
#include <sstream>


std::vector<HAProxyStats> StatsParser::parse(
    const std::string& raw
)
{
    std::vector<HAProxyStats> result;

    std::stringstream stream(raw);

    std::string line;


    while (std::getline(stream, line))
    {
        // 빈 줄 무시
        if (line.empty())
        {
            continue;
        }

        // HAProxy CSV 헤더 무시
        if (line[0] == '#')
        {
            continue;
        }


        std::stringstream row(line);

        std::string value;

        HAProxyStats stats;


        // pxname
        std::getline(
            row,
            stats.proxyName,
            ','
        );


        // svname
        std::getline(
            row,
            stats.serverName,
            ','
        );


        // addr
        std::getline(
            row,
            stats.address,
            ','
        );


        /*
         * HAProxy show stat CSV에서
         *
         * status는 뒤쪽 컬럼에 존재한다.
         *
         * 따라서 앞의 컬럼들을 건너뛴다.
         */

        for (int i = 0; i < 17; i++)
        {
            std::getline(
                row,
                value,
                ','
            );
        }


        std::getline(
            row,
            stats.status,
            ','
        );


        result.push_back(stats);
    }


    return result;
}