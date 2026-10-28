//! Errors a person may need to act on, kept free of secrets.

use totaldrop_proto::wire::RejectReason;

/// Everything that can end a transfer.
#[derive(Debug)]
pub enum Error {
    /// Key confirmation failed: the code was mistyped, or someone tried to
    /// intercept the transfer. The nameplate is now retired.
    WrongCode,
    /// The other side left before the transfer was set up.
    PeerLeft,
    /// The rendezvous server refused a request or could not be reached.
    Rendezvous(String),
    /// No network path to the other device worked.
    Connect(String),
    /// The other side broke the protocol.
    Protocol(&'static str),
    /// The offer was invalid (bad path, bad signature, over a limit).
    InvalidOffer(String),
    /// The receiver turned the offer down.
    Rejected(RejectReason),
    /// The other side cancelled.
    Cancelled(String),
    /// This side stopped the transfer (see [`crate::Stop`]).
    Stopped,
    /// A local file problem.
    Io(std::io::Error),
    /// The transfer code could not be read.
    Code(totaldrop_proto::CodeError),
    /// The operating system's random number generator failed.
    Random,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::WrongCode => f.write_str(
                "the code did not match: it was mistyped, or someone tried to intercept the transfer",
            ),
            Error::PeerLeft => f.write_str("the other side left"),
            Error::Rendezvous(m) => write!(f, "rendezvous server: {m}"),
            Error::Connect(m) => write!(f, "could not connect to the other device: {m}"),
            Error::Protocol(m) => write!(f, "protocol error: {m}"),
            Error::InvalidOffer(m) => write!(f, "invalid offer: {m}"),
            Error::Rejected(r) => write!(f, "the receiver declined ({r:?})"),
            Error::Cancelled(m) => write!(f, "cancelled by the other side: {m}"),
            Error::Stopped => f.write_str("stopped on this device"),
            Error::Io(e) => write!(f, "{e}"),
            Error::Code(e) => write!(f, "{e}"),
            Error::Random => f.write_str("the random number generator failed"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<totaldrop_crypto::Error> for Error {
    fn from(e: totaldrop_crypto::Error) -> Self {
        match e {
            totaldrop_crypto::Error::Random => Error::Random,
            totaldrop_crypto::Error::Inauthentic => {
                Error::Protocol("message failed authentication")
            }
            totaldrop_crypto::Error::Malformed => Error::Protocol("malformed message"),
        }
    }
}

impl From<totaldrop_proto::wire::WireError> for Error {
    fn from(_: totaldrop_proto::wire::WireError) -> Self {
        Error::Protocol("undecodable message")
    }
}

/// A side that stops closes the connection with reason `stopped` right
/// after its cancel message, and the close can overtake the message. Seen
/// that way, it is still the other side stopping.
fn closed_by_peer(e: &quinn::ConnectionError) -> Option<Error> {
    match e {
        quinn::ConnectionError::ApplicationClosed(c) if &c.reason[..] == b"stopped" => {
            Some(Error::Cancelled("stopped".into()))
        }
        _ => None,
    }
}

impl From<quinn::ConnectionError> for Error {
    fn from(e: quinn::ConnectionError) -> Self {
        closed_by_peer(&e).unwrap_or_else(|| Error::Connect(e.to_string()))
    }
}

impl From<quinn::WriteError> for Error {
    fn from(e: quinn::WriteError) -> Self {
        match e {
            quinn::WriteError::ConnectionLost(c) => c.into(),
            e => Error::Connect(e.to_string()),
        }
    }
}

impl From<quinn::ReadExactError> for Error {
    fn from(e: quinn::ReadExactError) -> Self {
        match e {
            quinn::ReadExactError::FinishedEarly(_) => Error::Protocol("stream ended early"),
            quinn::ReadExactError::ReadError(e) => e.into(),
        }
    }
}

impl From<quinn::ReadError> for Error {
    fn from(e: quinn::ReadError) -> Self {
        match e {
            quinn::ReadError::ConnectionLost(c) => c.into(),
            e => Error::Connect(e.to_string()),
        }
    }
}

/// Short for `Result<T, Error>`.
pub type Result<T> = core::result::Result<T, Error>;
