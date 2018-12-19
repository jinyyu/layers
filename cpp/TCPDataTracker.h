#ifndef LAYERS_TCPDATATRACKER_H
#define LAYERS_TCPDATATRACKER_H
#include <stdint.h>
#include <cstddef>
#ifdef __cplusplus
extern "C"
{
#endif

typedef void(* on_data_callback)(const void* ctx, const char* data, uint32_t len);

void* new_tcp_data_tracker(uint32_t seq);

void tcp_data_tracker_set_callback(void* tracker, void* ctx, on_data_callback cb);

void tcp_data_tracker_update_seq(void* tracker, uint32_t seq);

void tcp_data_tracker_process_data(void* tracker, uint32_t seq, const char* data, uint32_t len);

void free_tcp_data_tracker(void* tracker);


#ifdef __cplusplus
}
#endif

#endif //LAYERS_TCPDATATRACKER_H
