#include "TCPDataTracker.h"
#include "debug_log.h"
#include <stdio.h>
#include <vector>
#include <map>

// As defined by RFC 1982 - 2 ^ (SERIAL_BITS - 1)
static const uint32_t k_seq_number_diff = 2147483648U;

static int seq_compare(uint32_t seq1, uint32_t seq2)
{
    if (seq1 == seq2) {
        return 0;
    }
    if (seq1 < seq2) {
        return (seq2 - seq1 < k_seq_number_diff) ? -1 : 1;
    }
    else {
        return (seq1 - seq2 > k_seq_number_diff) ? -1 : 1;
    }
}

typedef std::vector<uint8_t> Payload;

typedef std::map<uint32_t, Payload> BufferPayload;

class TCPDataTracker
{
public:
    explicit TCPDataTracker(uint32_t seq)
        : next_seq_(seq),
          ctx_(nullptr),
          cb_(nullptr)
    {
        //LOG_DEBUG("new");
    }

    ~TCPDataTracker()
    {
        //LOG_DEBUG("release");
    }

    void handle_data(uint32_t seq, const char* data, uint32_t len)
    {
        const uint32_t chunk_end = seq + len;
        // If the end of the chunk ends before current sequence number, ignore it.
        if (seq_compare(chunk_end, next_seq_) < 0) {
            return;
        }
        // If it starts before our sequence number, slice it
        if (seq_compare(seq, next_seq_) < 0) {
            const uint32_t diff = next_seq_ - seq;
            data += diff;
            len -= diff;
            seq = next_seq_;
        }
        if (seq == next_seq_) {
            cb_(ctx_, data, len);
            next_seq_ += len;

        } else {
            // Store this payload
            store_payload(seq, data, len);
        }


        // Keep looping while the fragments seq is lower or equal to our seq
        auto it = buffer_payload_.find(next_seq_);
        while (it != buffer_payload_.end() && seq_compare(it->first, next_seq_) <= 0) {
            // Does this fragment start before our sequence number?
            if (seq_compare(it->first, next_seq_) < 0) {
                uint32_t fragment_end = it->first + static_cast<uint32_t>(it->second.size());
                int comparison = seq_compare(fragment_end, next_seq_);
                // Does it end after our sequence number?
                if (comparison > 0) {
                    // Then slice it
                    std::vector<uint8_t>& payload = it->second;
                    // First update this counter

                    uint32_t diff = next_seq_ - it->first;
                    const uint8_t* data = payload.data() + diff;
                    uint32_t len = payload.size() - diff;

                    store_payload(next_seq_, (const char*) data, len);
                    it = erase_iterator(it);
                }
                else {
                    // Otherwise, we've seen this part of the payload. Erase it.
                    it = erase_iterator(it);
                }
            }
            else {
                cb_(ctx_, (const char*) it->second.data(), static_cast<uint32_t>(it->second.size()));
                next_seq_ += it->second.size();
                it = erase_iterator(it);
            }
        }

    }

    void store_payload(uint32_t seq, const char* data, uint32_t len)
    {
        auto it = buffer_payload_.find(seq);
        if (it == buffer_payload_.end()) {
            Payload payload(data, data + len);
            buffer_payload_[seq] = std::move(payload);
        }
        else if (it->second.size() < len) {
            Payload payload(data, data + len);
            it->second = std::move(payload);
        }
    }

    BufferPayload::iterator erase_iterator(BufferPayload::iterator iter)
    {
        auto output = iter;
        ++output;
        buffer_payload_.erase(iter);
        if (output == buffer_payload_.end()) {
            output = buffer_payload_.begin();
        }
        return output;
    }

    void set_callback(void* ctx, on_data_callback cb)
    {
        ctx_ = ctx;
        cb_ = cb;
    }

    void update_seq(uint32_t seq)
    {
        next_seq_ = seq;
    }

private:
    uint32_t next_seq_;
    void* ctx_;
    on_data_callback cb_;
    BufferPayload buffer_payload_;
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

void tcp_data_tracker_update_seq(void* tracker, uint32_t seq)
{
    ((TCPDataTracker*) tracker)->update_seq(seq);
}

void tcp_data_tracker_process_data(void* tracker, uint32_t seq, const char* data, uint32_t len)
{
    ((TCPDataTracker*) tracker)->handle_data(seq, data, len);
}


