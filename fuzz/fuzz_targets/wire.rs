#![no_main]
//! Every wire decoder must accept or reject any input without panicking, and
//! anything it accepts must re-encode to bytes that decode to the same value.

use libfuzzer_sys::fuzz_target;
use totaldrop_proto::wire::{
    Certificate, ClientMessage, Control, DataHeader, Hints, LengthDecoder, ServerMessage,
};

fuzz_target!(|data: &[u8]| {
    if let Ok(m) = ClientMessage::decode(data) {
        assert_eq!(ClientMessage::decode(&m.encode()).ok(), Some(m));
    }
    if let Ok(m) = ServerMessage::decode(data) {
        assert_eq!(ServerMessage::decode(&m.encode()).ok(), Some(m));
    }
    if let Ok(m) = Control::decode(data) {
        assert_eq!(Control::decode(&m.encode()).ok(), Some(m));
    }
    if let Ok(m) = Certificate::decode(data) {
        assert_eq!(Certificate::decode(&m.encode()).ok(), Some(m));
    }
    if let Ok(m) = Hints::decode(data) {
        assert_eq!(Hints::decode(&m.encode()).ok(), Some(m));
    }
    if let Ok(m) = DataHeader::decode(data) {
        assert_eq!(DataHeader::decode(&m.encode()).ok(), Some(m));
    }
    let mut d = LengthDecoder::new(1 << 20);
    for b in data {
        match d.push(*b) {
            Ok(None) => continue,
            _ => break,
        }
    }
});
