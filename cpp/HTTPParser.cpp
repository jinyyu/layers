#include "HTTPParser.h"
#include "http_parser.h"
#include "debug_log.h"


static http_parser_settings g_request;

static http_parser_settings g_response;

class HTTPParser
{
public:
    explicit HTTPParser(void* ctx)
    {
        http_parser_init(&request_parser_, HTTP_REQUEST);
        request_parser_.data = ctx;

        http_parser_init(&response_parser_, HTTP_RESPONSE);
        response_parser_.data = ctx;
    }

    size_t execute_request(const char* data, size_t len)
    {
        return http_parser_execute(&request_parser_, &g_request, data, len);
    }

    size_t execute_response(const char* data, size_t len)
    {
        LOG_DEBUG("===================%s", data);
        return http_parser_execute(&request_parser_, &g_response, data, len);
    }

    ~HTTPParser()
    {

    }

private:
    http_parser request_parser_;
    http_parser response_parser_;

};

void init_http_parser_setting(struct http_parser_settings request, struct http_parser_settings response)
{
    LOG_DEBUG("init http parser");
    memcpy(&g_request, &request, sizeof(request));
    memcpy(&g_response, &response, sizeof(response));
}

void* new_http_parser(void* ctx)
{
    return new HTTPParser(ctx);
}

size_t http_parser_execute_request(void* parser, const char* data, size_t len)
{
    return ((HTTPParser*) parser)->execute_request(data, len);
}

size_t http_parser_execute_response(void* parser, const char* data, size_t len)
{
    return ((HTTPParser*) parser)->execute_response(data, len);
}

void free_http_parser(void* parser)
{
    delete ((HTTPParser*) parser);
}


