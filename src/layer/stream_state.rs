pub const STATE_NONE: u32 = (0);
pub const STATE_PROTOCOL_DETECTING: u32 = (1 << 0);
pub const STATE_PROTOCOL_SUCCESS: u32 = (1 << 1);
pub const STATE_PROTOCOL_FAILED: u32 = (1 << 2);
pub const STATE_PROTOCOL_FINISHED: u32 = (STATE_PROTOCOL_DETECTING | STATE_PROTOCOL_SUCCESS);
pub const STATE_PROTOCOL_ALL: u32 = (STATE_PROTOCOL_DETECTING | STATE_PROTOCOL_FINISHED);
pub const STATE_STREAM_FINISHED: u32 = (1 << 3);
pub const STATE_STREAM_SKIP: u32 = (1 << 4);

pub fn state_to_string(state: u32) -> String {
    let mut ret = String::new();

    if state & STATE_PROTOCOL_DETECTING > 0 {
        ret.push_str("detecting,")
    }
    if state & STATE_PROTOCOL_SUCCESS > 0 {
        ret.push_str("detect_success,")
    }
    if state & STATE_PROTOCOL_FAILED > 0 {
        ret.push_str("detect_failed,")
    }
    if state & STATE_STREAM_FINISHED > 0 {
        ret.push_str("finished,")
    }
    if state & STATE_STREAM_SKIP > 0 {
        ret.push_str("skip,")
    }
    if ret.is_empty() {
        return "none".to_string();
    }
    ret.pop();
    return ret;
}
