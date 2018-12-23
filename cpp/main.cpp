#include <stdio.h>
#include "http_parser.h"
#include <signal.h>

int main(int argc, char* argv[])
{
    fprintf(stderr, "%d", sizeof(http_errno));
}