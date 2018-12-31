#include "MimeMessage.h"
#include <string>
#include <memory>
#include <vector>
#include "debug_log.h"


class MimeMessage;

typedef std::shared_ptr<MimeMessage> MimeMessagePtr;

class MimeMessage
{
public:
    explicit MimeMessage(GMimeObject* obj)
        : is_text_(false),
          filename_(nullptr),
          content_(g_mime_object_get_content_type(obj)),
          out_array_(nullptr),
          out_stream_(nullptr),
          obj_(obj)
    {
        g_assert(obj_);
        g_object_ref(obj_);
    }

    ~MimeMessage()
    {
        g_object_unref(obj_);
        if (out_stream_) {
            g_object_unref(out_stream_);
        }
    }

    void parse_recursive()
    {
        if (GMIME_IS_MULTIPART(obj_)) {
            GMimeMultipart* multipart = (GMimeMultipart*) obj_;
            for (int i = 0; i < g_mime_multipart_get_count(multipart); ++i) {
                GMimeObject* subpart = g_mime_multipart_get_part(multipart, i);
                MimeMessagePtr child(new MimeMessage(subpart));
                child->parse_recursive();
                sub_messages_.push_back(child);
            }
        }
        else if (GMIME_IS_PART(obj_)) {
            GMimePart* part = (GMimePart*) obj_;

            GMimeContentEncoding encoding = g_mime_part_get_content_encoding(part);
            filename_ = g_mime_part_get_filename(part);

            alloc_out_stream();

            if (GMIME_IS_TEXT_PART(obj_)) {
                //text
                is_text_ = true;
                GMimeTextPart* part = (GMimeTextPart*) obj_;
                char* text = g_mime_text_part_get_text(part);
                if (!text) {
                    return;
                }
                g_byte_array_append(out_array_, (const guint8*) text, strlen(text));
                g_free(text);
            }
            else {
                //binary
                GMimeDataWrapper* content = g_mime_part_get_content(part);
                if (!content) {
                    return;
                }

                GMimeStream* in_stream = g_mime_data_wrapper_get_stream(content);
                GMimeStream* filter_stream = g_mime_stream_filter_new(out_stream_);
                GMimeFilter* filter = g_mime_filter_basic_new(encoding, false); //decode
                g_mime_stream_filter_set_owner((GMimeStreamFilter*) filter_stream, true);
                g_mime_stream_filter_add((GMimeStreamFilter*) filter_stream, filter);
                g_object_unref(filter);
                if (g_mime_stream_write_to_stream(in_stream, filter_stream) == -1) {
                    LOG_DEBUG("g_mime_stream_write_to_stream error %s", g_strerror(errno));
                }
                g_object_unref(filter_stream);
            }
        }
    }

    void walk(mime_message_walk_callback callback, void* user)
    {
        if (out_array_ && out_array_->len > 0) {
            callback((const char*) out_array_->data, out_array_->len, is_text_, filename_, content_, user);
        }
        for (size_t i = 0; i < sub_messages_.size(); ++i) {
            sub_messages_[i]->walk(callback, user);
        }
    }

private:
    void alloc_out_stream()
    {
        if (!out_array_) {
            out_array_ = g_byte_array_new();
            out_stream_ = g_mime_stream_mem_new_with_byte_array(out_array_);
        }
    }

    //options
    bool is_text_;
    const char* filename_;
    GMimeContentType* content_;

    GByteArray* out_array_;
    GMimeStream* out_stream_;
    GMimeObject* obj_;
    std::vector<MimeMessagePtr> sub_messages_;
};

void* new_mime_message(GMimeObject* obj)
{
    MimeMessage* msg = new MimeMessage(obj);
    msg->parse_recursive();
    return (void*) msg;
}

void delete_mime_message(void* msg)
{
    delete ((MimeMessage*) msg);
}

void mime_message_walk(void* msg, mime_message_walk_callback callback, void* user)
{
    MimeMessage* m = (MimeMessage*) msg;
    m->walk(callback, user);

}


