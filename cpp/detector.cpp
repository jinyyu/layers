#include "detector.h"
#include <ndpi/ndpi_api.h>

void* alloc_ndpi()
{
    ndpi_detection_module_struct* handle = ndpi_init_detection_module();
    NDPI_PROTOCOL_BITMASK all;
    NDPI_BITMASK_SET_ALL(all);
    ndpi_set_protocol_detection_bitmask2(handle, &all);
    return (void*)handle;
}

uint32_t ndpi_flow_struct_size()
{
    return sizeof(struct ndpi_flow_struct);
}