#include "MimeParser.h"
#include "Slice.h"
#include "debug_log.h"

#include <gmime/gmime.h>
#include <memory>
#include <vector>
#include <string>


static std::string get_display_name(InternetAddressList* addr_list)
{
    std::string ret;
    if (!addr_list) {
        return ret;
    }

    int len = internet_address_list_length(addr_list);
    for (int i = 0; i < len; ++i) {
        InternetAddress* addr = internet_address_list_get_address(addr_list, i);
        char* display = internet_address_to_string(addr, NULL, true);
        if (!display) {
            continue;
        }
        if (!ret.empty()) {
            ret.push_back(',');
        }
        ret.append(display);
        free(display);
    }
    return ret;
}

class MimeMessage;

typedef std::unique_ptr<MimeMessage> MimeMessagePtr;

typedef std::function<void(MimeMessage* message, const Slice& slice)> MimeWalkCallback;

class MimeParser;

class MimeMessage
{
public:
    MimeMessage(MimeParser* parser, GMimeObject* obj)
        : is_text_(false),
          parser_(parser),
          array_out_(nullptr),
          out_stream_(nullptr),
          obj_(obj)
    {
        g_assert(obj_);
        g_object_ref(obj_);
    }

    ~MimeMessage()
    {
        g_object_unref(obj_);
        if (out_stream_) g_object_unref(out_stream_);
    }

    void parse_recursive()
    {
        //parse_headers();
        if (GMIME_IS_MULTIPART(obj_)) {
            GMimeMultipart* multipart = (GMimeMultipart*) obj_;

            for (int i = 0; i < g_mime_multipart_get_count(multipart); ++i) {
                GMimeObject* subpart = g_mime_multipart_get_part(multipart, i);
                MimeMessagePtr child(new MimeMessage(parser_, subpart));
                child->parse_recursive();
                sub_messages_.push_back(std::move(child));
            }
        }
        else if (GMIME_IS_PART(obj_)) {
            GMimePart* part = (GMimePart*) obj_;

            GMimeContentEncoding encoding = g_mime_part_get_content_encoding(part);
            const char* filename = g_mime_part_get_filename(part);
            if (filename) {
                filename_ = filename;
                LOG_DEBUG("filename %s", filename);
            }

            alloc_out_stream();

            if (GMIME_IS_TEXT_PART(obj_)) {
                //text
                is_text_ = true;
                GMimeTextPart* part = (GMimeTextPart*) obj_;
                char* text = g_mime_text_part_get_text(part);
                if (!text) {
                    return;
                }
                g_byte_array_append(array_out_, (const guint8*) text, strlen(text));
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

    void walk_recursive(const MimeWalkCallback& callback)
    {
        if (array_out_ && array_out_->len > 0) {
            Slice slice((const char*) array_out_->data, array_out_->len);
            callback(this, slice);
        }
        for (size_t i = 0; i < sub_messages_.size(); ++i) {
            sub_messages_[i]->walk_recursive(callback);
        }
    }

    const std::string& filename()
    {
        return filename_;
    }

    bool is_text() const
    {
        return is_text_;
    }

    MimeParser* parser() const
    {
        return parser_;
    }

private:
    void alloc_out_stream()
    {
        if (!array_out_) {
            array_out_ = g_byte_array_new();
            out_stream_ = g_mime_stream_mem_new_with_byte_array(array_out_);
        }
    }


private:
    //options
    bool is_text_;
    std::string filename_;

    MimeParser* parser_;
    GByteArray* array_out_; //out
    GMimeStream* out_stream_;
    GMimeObject* obj_;
    std::vector<MimeMessagePtr> sub_messages_;
};

class MimeParser
{
public:
    explicit MimeParser(GMimeStream* in_stream)
        : in_stream_(in_stream)
    {
        g_object_ref(in_stream_);
    }

    ~MimeParser()
    {
        g_object_unref(in_stream_);

    }

    void parse()
    {
        GMimeFormat format = GMIME_FORMAT_MESSAGE;
        GMimeParser* parser = g_mime_parser_new_with_stream(in_stream_);
        g_mime_parser_set_format(parser, format);

        GMimeMessage* msg = g_mime_parser_construct_message(parser, NULL);
        if (msg && msg->mime_part) {
            MimeMessagePtr root(new MimeMessage(this, msg->mime_part));
            root_ = std::move(root);

            const char* subject = g_mime_message_get_subject(msg);
            if (subject) {
                email_subject = subject;
            }

            InternetAddressList* from = g_mime_message_get_from(msg);
            email_from = get_display_name(from);

            InternetAddressList* sender = g_mime_message_get_sender(msg);
            email_sender = get_display_name(sender);

            InternetAddressList* reply = g_mime_message_get_reply_to(msg);
            email_reply_to = get_display_name(reply);

            InternetAddressList* to = g_mime_message_get_to(msg);
            email_to = get_display_name(to);

            InternetAddressList* cc = g_mime_message_get_cc(msg);
            email_cc = get_display_name(cc);

            InternetAddressList* bcc = g_mime_message_get_bcc(msg);
            email_bcc = get_display_name(bcc);


            root_->parse_recursive();
            g_object_unref(msg);
        }
        g_object_unref(parser);

    }

    void walk(const MimeWalkCallback& callback)
    {
        if (root_) {
            root_->walk_recursive(callback);
        }
    }

public:
    //email option
    std::string email_subject;
    std::string email_from;
    std::string email_sender;
    std::string email_reply_to;
    std::string email_to;
    std::string email_cc;
    std::string email_bcc;

private:
    GMimeStream* in_stream_;
    MimeMessagePtr root_;
};

class MimeParserInitHelper
{
public:
    explicit MimeParserInitHelper()
    {
        LOG_DEBUG("init mime parser");
        g_mime_init();
    }

    ~MimeParserInitHelper()
    {
        g_mime_shutdown();
    }
};

static MimeParserInitHelper __init;


