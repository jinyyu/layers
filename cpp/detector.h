#ifndef LAYERS_DISTRIBUTION_DETECTOR_H
#define LAYERS_DISTRIBUTION_DETECTOR_H
#ifdef __cplusplus
extern "C"
{
#endif


void * init_ndpi_ctx();

void free_ndpi_ctx(void* ctx);

void* new_ndpi_flow();

void free_ndpi_flow(void* flow);

void* new_ndpi_flow_id();

void free_ndpi_flow_id(void* id);


#ifdef __cplusplus
}
#endif

#endif //LAYERS_DISTRIBUTION_DETECTOR_H
