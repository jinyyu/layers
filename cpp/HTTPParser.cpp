#include "HTTPParser.h"
#include "http_parser.h"
#include "debug_log.h"


#define PARSE_OK (0)
#define PARSE_ERROR (1)

class HTTPParser
{
public:
    explicit HTTPParser()
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

class HttpInitHelper
{
public:
    explicit HttpInitHelper()
    {
        LOG_DEBUG("init http parser");
        g_request.on_message_begin = on_request_message_begin;
        g_request.on_url = on_url;
        g_request.on_status = nullptr;
        g_request.on_header_field = on_request_header_field;
        g_request.on_header_value = on_request_header_value;
        g_request.on_headers_complete = on_request_headers_complete;
        g_request.on_body = on_request_body;
        g_request.on_message_complete = on_request_message_complete;
        g_request.on_chunk_header = nullptr;
        g_request.on_chunk_complete = nullptr;

        g_response.on_message_begin = on_response_message_begin;
        g_response.on_url = nullptr;
        g_response.on_status = on_status;
        g_response.on_header_field = on_response_header_field;
        g_response.on_header_value = on_response_header_value;
        g_response.on_headers_complete = on_response_headers_complete;
        g_response.on_body = on_response_body;
        g_response.on_message_complete = on_response_message_complete;
        g_response.on_chunk_header = nullptr;
        g_response.on_chunk_complete = nullptr;
    }
};

static HttpInitHelper once;

void* new_http_parser()
{

}
