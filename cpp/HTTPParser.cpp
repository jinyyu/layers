#include "HTTPParser.h"
#include "http_parser.h"
#include "debug_log.h"


#define PARSE_OK (0)
#define PARSE_ERROR (1)

class HTTPParser
{
public:
    explicit HTTPParser(void* ctx)
        : ctx_(ctx)
    {
        memset(&request_parser_, 0, sizeof(request_parser_));
        http_parser_init(&request_parser_, HTTP_REQUEST);
        request_parser_.data = this;

        memset(&response_parser_, 0, sizeof(response_parser_));
        http_parser_init(&response_parser_, HTTP_RESPONSE);
        response_parser_.data = this;
    }

    ~HTTPParser()
    {

    }


private:
    void* ctx_;
    http_parser request_parser_;
    http_parser response_parser_;

};

static int on_request_message_begin(http_parser*)
{
    return PARSE_OK;
}

static int on_url(http_parser*, const char* at, size_t length)
{
    return PARSE_OK;
}

static int on_request_header_field(http_parser*, const char* at, size_t length)
{
    return PARSE_OK;
}

static int on_request_header_value(http_parser*, const char* at, size_t length)
{
    return PARSE_OK;
}

static int on_request_headers_complete(http_parser*)
{
    return PARSE_OK;
}

static int on_request_body(http_parser*, const char* at, size_t length)
{
    return PARSE_OK;
}

static int on_request_message_complete(http_parser*)
{
    return PARSE_OK;
}

static int on_response_message_begin(http_parser*)
{
    return PARSE_OK;
}

static int on_status(http_parser*, const char* at, size_t length)
{
    return PARSE_OK;
}

static int on_response_header_field(http_parser*, const char* at, size_t length)
{
    return PARSE_OK;
}

static int on_response_header_value(http_parser*, const char* at, size_t length)
{
    return PARSE_OK;
}

static int on_response_headers_complete(http_parser*)
{
    return PARSE_OK;
}

static int on_response_body(http_parser*, const char* at, size_t length)
{
    return PARSE_OK;
}

static int on_response_message_complete(http_parser*)
{
    return PARSE_OK;
}

static http_parser_settings g_request;

static http_parser_settings g_response;

void init_http_parser_setting(struct http_parser_settings request, struct http_parser_settings response)
{
    g_request = request;
    g_response = response;
}

void* new_http_parser(void* ctx)
{
    return new HTTPParser(ctx);
}

void free_http_parser(void* parser)
{
    delete ((HTTPParser*) parser);
}


