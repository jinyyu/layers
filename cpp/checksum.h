#ifndef LAYERS_DISTRIBUTION_CHECKSUM_H
#define LAYERS_DISTRIBUTION_CHECKSUM_H
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

uint16_t layers_checksum(const char *buf, unsigned size);

#ifdef __cplusplus
}
#endif


#endif //LAYERS_DISTRIBUTION_CHECKSUM_H
