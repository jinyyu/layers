#ifndef LAYERS_HTTPPARSER_H
#define LAYERS_HTTPPARSER_H
#include "http_parser.h"

#ifdef __cplusplus
extern "C"
{
#endif

void init_http_parser_setting(struct http_parser_settings request, struct http_parser_settings response);

void* new_http_parser(void* ctx);

size_t http_parser_execute_request(void* parser, const char* data, size_t len);

size_t http_parser_execute_response(void* parser, const char* data, size_t len);

void free_http_parser(void* parser);


#ifdef __cplusplus
}
#endif

#endif //LAYERS_HTTPPARSER_H
