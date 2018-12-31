#ifndef LAYERS_MIMEMESSAGE_H
#define LAYERS_MIMEMESSAGE_H
#include <gmime/gmime.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C"
{
#endif


typedef void
(* mime_message_walk_callback)(const char* data, uint32_t len, bool is_text, const char* filename, GMimeContentType* content, void* user);

void* new_mime_message(GMimeObject* obj);

void delete_mime_message(void*);

void mime_message_walk(void* msg, mime_message_walk_callback callback, void* user);

#ifdef __cplusplus
}
#endif

#endif //LAYERS_MIMEMESSAGE_H
