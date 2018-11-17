#include <stdio.h>
#include <time.h>

int main(int argc, char* argv[])
{
    struct timeval ts;
    fprintf(stderr, "%lu-%lu\n", sizeof(ts.tv_sec), sizeof(ts.tv_usec));
}