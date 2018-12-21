#include <stdio.h>
#include "http_parser.h"

int main(int argc, char* argv[])
{
    fprintf(stderr, "%d, %d", sizeof(HTTP_REQUEST), HTTP_REQUEST);
}