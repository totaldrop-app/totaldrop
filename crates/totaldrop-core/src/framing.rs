//! Length-framed messages on QUIC streams (spec section 2).

use totaldrop_proto::wire::{LengthDecoder, encode_length};

use crate::error::{Error, Result};

/// Write one frame: LEB128 length, then the bytes.
pub async fn write_frame(send: &mut quinn::SendStream, data: &[u8]) -> Result<()> {
    let mut buf = encode_length(data.len());
    buf.extend_from_slice(data);
    send.write_all(&buf).await?;
    Ok(())
}

/// Read one frame of at most `max` bytes. Returns `None` if the stream
/// ended cleanly before a new frame began.
pub async fn read_frame(recv: &mut quinn::RecvStream, max: usize) -> Result<Option<Vec<u8>>> {
    let mut decoder = LengthDecoder::new(max);
    let mut first = true;
    let len = loop {
        let mut byte = [0u8; 1];
        match recv.read(&mut byte).await? {
            None if first => return Ok(None),
            None => return Err(Error::Protocol("stream ended inside a frame")),
            Some(0) => continue,
            Some(_) => {}
        }
        first = false;
        let [b] = byte;
        if let Some(len) = decoder
            .push(b)
            .map_err(|_| Error::Protocol("frame too large"))?
        {
            break len;
        }
    };
    let mut data = vec![0u8; len];
    recv.read_exact(&mut data).await?;
    Ok(Some(data))
}

/// Read one frame that must be present.
pub async fn expect_frame(recv: &mut quinn::RecvStream, max: usize) -> Result<Vec<u8>> {
    read_frame(recv, max)
        .await?
        .ok_or(Error::Protocol("stream ended early"))
}
