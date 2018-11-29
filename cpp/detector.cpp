#include "detector.h"
#include <ndpi/ndpi_api.h>

void* init_ndpi_ctx()
{
    ndpi_detection_module_struct* handle = ndpi_init_detection_module();
    NDPI_PROTOCOL_BITMASK all;
    NDPI_BITMASK_SET_ALL(all);
    ndpi_set_protocol_detection_bitmask2(handle, &all);
    return (void*) handle;
}

void free_ndpi_ctx(void* ctx)
{
    ndpi_exit_detection_module((struct ndpi_detection_module_struct*) ctx);
}

void* new_ndpi_flow()
{
    void* ret = ndpi_flow_malloc(SIZEOF_FLOW_STRUCT);
    memset(ret, 0, SIZEOF_FLOW_STRUCT);
    return ret;
}

void free_ndpi_flow(void* flow)
{
    ndpi_flow_free(flow);
}

void* new_ndpi_flow_id()
{
    void* ret = ndpi_malloc(SIZEOF_ID_STRUCT);
    memset(ret, 0, sizeof(SIZEOF_ID_STRUCT));
    return ret;
}

void free_ndpi_flow_id(void* id)
{
    ndpi_free(id);
}