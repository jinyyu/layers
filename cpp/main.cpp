#include <stdio.h>
#include "http_parser.h"
#include <signal.h>
#include <arpa/inet.h>
#include <string.h>


void test()
{
    short num = 0x1122;
    char* c;
    c = (char*) &num;
    if (*c == 0x22)
        printf("this is little end\n");
    else
        printf("this is big end\n");

}

int main(int argc, char* argv[])
{
    test();

    uint8_t buffer[4];
    inet_pton(AF_INET, "0.0.0.1", &buffer);
    fprintf(stderr, "%d.%d.%d.%d\n", buffer[0], buffer[1], buffer[2], buffer[3]);

    uint32_t a = 0;
    memcpy(&a, buffer, 4);
    fprintf(stderr, "==========================%d\n", memcmp(&a, buffer, 4));

    fprintf(stderr, "%u", a);

}