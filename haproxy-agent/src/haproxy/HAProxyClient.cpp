#include "haproxy/HAProxyClient.hpp"

#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

#include <cstring>
#include <iostream>


HAProxyClient::HAProxyClient(
    const std::string& path
)
    : socketPath(path)
{

}


HAProxyClient::~HAProxyClient()
{

}



bool HAProxyClient::connect()
{
    int fd = socket(
        AF_UNIX,
        SOCK_STREAM,
        0
    );


    if(fd < 0)
    {
        return false;
    }


    sockaddr_un addr{};

    addr.sun_family = AF_UNIX;


    strncpy(
        addr.sun_path,
        socketPath.c_str(),
        sizeof(addr.sun_path) - 1
    );


    int result = ::connect(
        fd,
        (sockaddr*)&addr,
        sizeof(addr)
    );


    close(fd);


    return result == 0;
}





std::string HAProxyClient::execute(
    const std::string& command
)
{
    int fd = socket(
        AF_UNIX,
        SOCK_STREAM,
        0
    );


    if(fd < 0)
    {
        return "";
    }



    sockaddr_un addr{};

    addr.sun_family = AF_UNIX;


    strncpy(
        addr.sun_path,
        socketPath.c_str(),
        sizeof(addr.sun_path) - 1
    );



    if(
        ::connect(
            fd,
            (sockaddr*)&addr,
            sizeof(addr)
        ) < 0
    )
    {
        close(fd);
        return "";
    }



    send(
        fd,
        command.c_str(),
        command.size(),
        0
    );



    char buffer[8192];

    std::string result;



    int n;


    while(
        (n = recv(
            fd,
            buffer,
            sizeof(buffer) - 1,
            0
        )) > 0
    )
    {
        buffer[n] = '\0';

        result += buffer;
    }



    close(fd);


    return result;
}





std::string HAProxyClient::getInfo()
{
    return execute(
        "show info\n"
    );
}





std::string HAProxyClient::getStats()
{
    return execute(
        "show stat\n"
    );
}