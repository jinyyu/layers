#ifndef LAYERS_DEBUG_LOG_H
#define LAYERS_DEBUG_LOG_H
#include <string.h>
#include <stdio.h>
#include <string>

#define LOG_DEBUG(format, ...) do { fprintf(stderr, "[DEBUG] [%s:%d] " format "\n", strrchr(__FILE__, '/') + 1, __LINE__, ##__VA_ARGS__); } while(0)

#endif //LAYERS_DEBUG_LOG_H
