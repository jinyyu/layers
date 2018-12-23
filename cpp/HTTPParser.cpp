#include "HTTPParser.h"


const char* http_errno_description_from_parser(http_parser* parser)
{
    return http_errno_description((http_errno) parser->http_errno);
}


