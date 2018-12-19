#include <map>
#include <stdio.h>
#include "http_parser.h"

struct Parser {
    /** PRIVATE **/
    uint32_t abc;

    uint32_t nread;          /* # bytes read in various scenarios */
    uint64_t content_length; /* # bytes in body (0 if no Content-Length header) */

    /** READ-ONLY **/
    unsigned short http_major;
    unsigned short http_minor;
    uint32_t def;

    /** PUBLIC **/
    void *data; /* A pointer to get hook to the "connection" or "socket" object */
};


int main()
{
    fprintf(stderr, "%d", sizeof(Parser));
}