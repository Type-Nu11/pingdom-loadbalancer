#include <iostream>

#include "haproxy/HAProxyClient.hpp"

int main() {
    std::cout << "- HAProxy Agent -" << std::endl;

    HAProxyClient client;

    if (!client.connect()) {
        std::cerr
            << "Failed to connect to HAProxy socket." 
            << std::endl;

        return 1;
    }

    // string a = "wertyui";
    // string b = a;

    // int arr[10] = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10};
    // std::cout << arr << std::endl;
    // std::cout << &arr[0] << std::endl;
    // std::cout << &arr << std::endl;

    std::cout
        << "[INFO] Connected"
        << std::endl;

    std::string info = client.getInfo();

    std::cout
        << "\n===== HAProxy Info =====\n" 
        << info
        << std::endl;


    return 0;
}


// int a = 10; // 10

// cout << a << endl; // 10
// cout << &a << endl; // 100

// int* ptr = &a;
// cout << ptr << endl; // 100
// cout << *ptr << endl; // 10
// cout << &ptr << endl; // 200

// int** ptr2 = &ptr;
// cout << ptr2 << endl; // 200
// cout << *ptr2 << endl; // 100
// cout << **ptr2 << endl; // 10

// int*** ptr3 = &ptr2;


