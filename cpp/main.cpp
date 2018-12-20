#include <stdio.h>
#include <string.h>
#include <malloc.h>
#include "http_parser.h"

int on_message_begin1(http_parser *parser)
{
    printf("on_message_begin\n");
    return 0;
}

int fake(http_parser *parser) {
    return 0;
}

int fake2(http_parser *parser, const char *at, size_t length) {
    return 0;
}

int on_message_complete1(http_parser *parser)
{
    printf("on_message_complete\n");
    return 0;
}

int main(int argc, char* argv[])
{
    const char *input =
        "HTTP/1.1 200 OK\r\n"
        "Content-Length: 10\r\n"
        "Content-Type: text/numbers\r\n"
        "\r\n"
        "0123456789"
        "HTTP/1.1 200 OK\r\n"
        "Content-Type: text/numbers\r\n"
        "Conte";

    const char *input2 =
        "nt-Length: 0\r\n"
        "\r\n"
        "HTTP/1.1 200 OK\r\n"
        "Content-Type: text/numbers\r\n"
        "Content-Length: 10\r\n"
        "\r\n"
        "0123456789";

    http_parser parser;
    http_parser_settings settings;
    settings.on_message_begin = on_message_begin1;
    settings.on_headers_complete  = fake;
    settings.on_message_complete = on_message_complete1;

    // Data callbacks: on_url, (common) on_header_field, on_header_value, on_body;
    settings.on_status            = fake2;
    settings.on_header_field      = fake2;
    settings.on_header_value      = fake2;
    settings.on_body              = fake2;

    http_parser_init(&parser, HTTP_RESPONSE);

    FILE* fp = fopen("/tmp/foo.txt", "r");
    char* buffer= (char*)malloc(10240);
    int n = fread(buffer, 1, 10240, fp);
    fprintf(stderr, "=============%d", n);

    size_t ret = http_parser_execute(&parser, &settings, buffer,n);
    enum http_errno err = HTTP_PARSER_ERRNO(&parser);
    printf("strlen=%zu ret=%zu errno=%d errstr=%s\n", strlen(input), ret, err,http_errno_name(err));




}