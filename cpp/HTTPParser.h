#ifndef LAYERS_HTTPPARSER_H
#define LAYERS_HTTPPARSER_H


#ifdef __cplusplus
extern "C"
{
#endif

void init_http_parser_setting(struct http_parser_settings request, struct http_parser_settings response);

void* new_http_parser(void* ctx);

void free_http_parser(void* parser);


#ifdef __cplusplus
}
#endif

#endif //LAYERS_HTTPPARSER_H
