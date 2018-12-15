#include "TCPDataTracker.h"
#include "debug_log.h"
#include <stdio.h>


class TCPDataTracker
{
public:
    explicit TCPDataTracker(uint32_t seq)
        : nex_seq_(seq),
          ctx_(nullptr),
          cb_(nullptr)
    {
        LOG_DEBUG("new");
    }

    ~TCPDataTracker()
    {
        LOG_DEBUG("release");

    }

    void handle_data(const char* data, uint32_t len)
    {
        LOG_DEBUG("handle data %u", len);

        cb_(ctx_, data, len);
    }

    void set_callback(void* ctx, on_data_callback cb)
    {
        ctx_ = ctx;
        cb_ = cb;
    }

private:
    uint32_t nex_seq_;
    void* ctx_;
    on_data_callback cb_;
};

void* new_tcp_data_tracker(uint32_t seq)
{
    return new TCPDataTracker(seq);
}

void free_tcp_data_tracker(void* tracker)
{
    delete ((TCPDataTracker*) tracker);

}

void tcp_data_tracker_set_callback(void* tracker, void* ctx, on_data_callback cb)
{
    ((TCPDataTracker*) tracker)->set_callback(ctx, cb);
}

void tcp_data_tracker_process_data(void* tracker, const char* data, uint32_t len)
{
    ((TCPDataTracker*) tracker)->handle_data(data, len);
}


