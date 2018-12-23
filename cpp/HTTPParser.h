#ifndef LAYERS_HTTP_PARSER_WAPPER_H
#define LAYERS_HTTP_PARSER_WAPPER_H
#include "http_parser.h"


#ifdef __cplusplus
extern "C"
{
#endif

const char* http_errno_description_from_parser(http_parser* parser);

#ifdef __cplusplus
}
#endif

#endif //LAYERS_HTTP_PARSER_WAPPER_H
