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

/// Why an error happened, in words the apps translate (#212). The
/// [`Display`](core::fmt::Display) text stays English and technical, for
/// problem reports; the cause is what a person reads in their language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cause {
    /// The connection stopped answering: a device lost its network or
    /// went to sleep.
    TimedOut,
    /// The connection to the other device was cut.
    ConnectionLost,
    /// No way to reach the other device worked.
    NoRoute,
    /// The relay could not be reached, refused, or is not set up.
    Relay,
    /// The other device sent something this version does not understand.
    Protocol,
    /// No space left on this device.
    DiskFull,
    /// TotalDrop may not read or write a file or folder.
    NoPermission,
    /// A file or folder is not there any more.
    NotFound,
    /// The folder is on a disk that cannot be written to.
    ReadOnly,
    /// A file changed while it was being sent.
    FileChanged,
    /// Nothing more to say than the error's own message.
    Unknown,
}

impl Cause {
    /// Every cause with something to say, for tests that each has a text.
    pub const ALL: [Cause; 10] = [
        Cause::TimedOut,
        Cause::ConnectionLost,
        Cause::NoRoute,
        Cause::Relay,
        Cause::Protocol,
        Cause::DiskFull,
        Cause::NoPermission,
        Cause::NotFound,
        Cause::ReadOnly,
        Cause::FileChanged,
    ];

    /// A stable name for the apps: `TimedOut`, `ConnectionLost`, … The
    /// translated sentence is the text `cause` + the name
    /// (`causeTimedOut`). Empty for [`Cause::Unknown`].
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Cause::TimedOut => "TimedOut",
            Cause::ConnectionLost => "ConnectionLost",
            Cause::NoRoute => "NoRoute",
            Cause::Relay => "Relay",
            Cause::Protocol => "Protocol",
            Cause::DiskFull => "DiskFull",
            Cause::NoPermission => "NoPermission",
            Cause::NotFound => "NotFound",
            Cause::ReadOnly => "ReadOnly",
            Cause::FileChanged => "FileChanged",
            Cause::Unknown => "",
        }
    }
}

/// Messages of [`Error::Connect`] that this crate gives, shared with
/// [`Error::cause`] so a reworded message cannot lose its cause unnoticed.
pub(crate) mod msg {
    pub(crate) const NO_CONNECTION: &str = "no connection to the other device";
    pub(crate) const RECEIVER_DID_NOT_CONNECT: &str = "the receiver did not connect";
    pub(crate) const NO_ADDRESS_ANNOUNCED: &str = "the other side announced no address";
    pub(crate) const NO_ADDRESS_ANSWERED: &str = "no address answered";
    pub(crate) const NO_SHARED_RELAY: &str =
        "no usable path: the transfer needs a relay both devices use";
    pub(crate) const PRIVACY_NEEDS_RELAY: &str = "privacy mode needs a relay";
    pub(crate) const INVALID_RELAY: &str = "invalid relay address";
    pub(crate) const RECEIVER_DID_NOT_ANSWER: &str = "the receiver did not answer";
    pub(crate) const SENDER_STOPPED_SENDING: &str = "the sender stopped sending";
    pub(crate) const ENDPOINT_CLOSED: &str = "endpoint closed";
}

/// The cause of an [`Error::Connect`] from its message: this crate's own
/// ([`msg`], and `relay…` from the relay link) and quinn's connection
/// errors (`timed out`, `reset by peer`, `closed by peer: …`).
fn connect_cause(m: &str) -> Cause {
    use msg::*;
    if m.starts_with("relay") || [INVALID_RELAY, PRIVACY_NEEDS_RELAY, NO_SHARED_RELAY].contains(&m)
    {
        Cause::Relay
    } else if m == "timed out" || m == SENDER_STOPPED_SENDING {
        Cause::TimedOut
    } else if m == "reset by peer" || m == ENDPOINT_CLOSED || m.starts_with("closed by peer") {
        Cause::ConnectionLost
    } else if [
        NO_CONNECTION,
        RECEIVER_DID_NOT_CONNECT,
        NO_ADDRESS_ANNOUNCED,
        NO_ADDRESS_ANSWERED,
    ]
    .contains(&m)
    {
        Cause::NoRoute
    } else {
        // Among others RECEIVER_DID_NOT_ANSWER: the connection was up, the
        // person just did not decide in time.
        Cause::Unknown
    }
}

/// The cause of an [`Error::Protocol`]: most say the other side broke the
/// protocol, but a few are this side's own limits or a stream that ended.
fn protocol_cause(m: &str) -> Cause {
    const CUT: [&str; 3] = [
        "stream ended early",
        "control stream ended",
        "stream closed",
    ];
    // Local failures and limits, and a message that failed authentication,
    // which may be an attack (docs/tehdit-modeli.md): an update would not
    // help, so nothing is suggested.
    const OWN: [&str; 9] = [
        "message failed authentication",
        "internal lock",
        "file too large",
        "chunk index",
        "chunk size",
        "bad file name",
        "no parent directory",
        "semaphore closed",
        "noise parameters",
    ];
    if CUT.contains(&m) {
        Cause::ConnectionLost
    } else if OWN.contains(&m) || m.ends_with("task failed") {
        Cause::Unknown
    } else {
        Cause::Protocol
    }
}

impl Error {
    /// Why this happened, for a translated explanation. Errors whose
    /// headline already says everything give [`Cause::Unknown`].
    #[must_use]
    pub fn cause(&self) -> Cause {
        use std::io::ErrorKind as K;
        match self {
            Error::Connect(m) => connect_cause(m),
            Error::Protocol(m) => protocol_cause(m),
            Error::Io(e) => match e.kind() {
                K::StorageFull | K::QuotaExceeded => Cause::DiskFull,
                K::PermissionDenied => Cause::NoPermission,
                K::NotFound => Cause::NotFound,
                K::ReadOnlyFilesystem => Cause::ReadOnly,
                _ if e.to_string().ends_with(CHANGED_AFTER_SCAN) => Cause::FileChanged,
                _ => Cause::Unknown,
            },
            _ => Cause::Unknown,
        }
    }
}

/// The end of the message for a file that changed after it was scanned.
pub(crate) const CHANGED_AFTER_SCAN: &str = "changed after it was scanned";

impl Error {
    /// The rendezvous server says the code's number is not in use: it was
    /// mistyped, has expired or was already used.
    #[must_use]
    pub fn is_code_not_in_use(&self) -> bool {
        matches!(self, Error::Rendezvous(r) if crate::rendezvous::CODE_NOT_IN_USE.contains(&r.as_str()))
    }

    /// The rendezvous server's address was refused before any connection.
    #[must_use]
    pub fn is_bad_server_address(&self) -> bool {
        matches!(self, Error::Rendezvous(r) if crate::rendezvous::BAD_ADDRESS.contains(&r.as_str()))
    }
}

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

/// Error code on a data stream the sender cut because it stopped
/// (`RESET_STREAM`, #193).
pub(crate) const STREAM_STOPPED: quinn::VarInt = quinn::VarInt::from_u32(1);

/// Error code on a data stream the sender cut because sending it failed.
pub(crate) const STREAM_FAILED: quinn::VarInt = quinn::VarInt::from_u32(2);

impl From<quinn::ReadError> for Error {
    fn from(e: quinn::ReadError) -> Self {
        match e {
            quinn::ReadError::ConnectionLost(c) => c.into(),
            // Only data streams are ever reset. A reset can overtake the
            // sender's cancel message; it says the same.
            quinn::ReadError::Reset(code) if code == STREAM_STOPPED => {
                Error::Cancelled("stopped".into())
            }
            quinn::ReadError::Reset(code) if code == STREAM_FAILED => {
                Error::Cancelled("the sender could not send a file".into())
            }
            e => Error::Connect(e.to_string()),
        }
    }
}

/// Short for `Result<T, Error>`.
pub type Result<T> = core::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    fn io(kind: std::io::ErrorKind) -> Error {
        Error::Io(std::io::Error::from(kind))
    }

    /// Every message this crate and quinn give a connection error has a
    /// cause the apps can translate (#212): before, a person read only
    /// "could not connect to the other device: timed out", in English.
    #[test]
    fn connection_errors_have_a_cause() {
        use quinn::ConnectionError as C;
        assert_eq!(Error::from(C::TimedOut).cause(), Cause::TimedOut);
        assert_eq!(Error::from(C::Reset).cause(), Cause::ConnectionLost);
        // This side closed it: nothing was cut.
        assert_eq!(Error::from(C::LocallyClosed).cause(), Cause::Unknown);
        let closed = C::ApplicationClosed(quinn::ApplicationClose {
            error_code: quinn::VarInt::from_u32(0),
            reason: b"failed"[..].into(),
        });
        assert_eq!(Error::from(closed).cause(), Cause::ConnectionLost);
        for m in [
            msg::NO_CONNECTION,
            msg::RECEIVER_DID_NOT_CONNECT,
            msg::NO_ADDRESS_ANNOUNCED,
            msg::NO_ADDRESS_ANSWERED,
        ] {
            assert_eq!(Error::Connect(m.into()).cause(), Cause::NoRoute, "{m}");
        }
        for m in [
            "relay: timed out",
            "relay refused the endpoint key",
            msg::INVALID_RELAY,
            msg::PRIVACY_NEEDS_RELAY,
            msg::NO_SHARED_RELAY,
        ] {
            assert_eq!(Error::Connect(m.into()).cause(), Cause::Relay, "{m}");
        }
        assert_eq!(
            Error::Connect(msg::SENDER_STOPPED_SENDING.into()).cause(),
            Cause::TimedOut
        );
        assert_eq!(
            Error::Connect(msg::ENDPOINT_CLOSED.into()).cause(),
            Cause::ConnectionLost
        );
        // The receiver may simply not have accepted yet.
        assert_eq!(
            Error::Connect(msg::RECEIVER_DID_NOT_ANSWER.into()).cause(),
            Cause::Unknown
        );
        assert_eq!(
            Error::Connect("something new".into()).cause(),
            Cause::Unknown
        );
    }

    #[test]
    fn file_and_other_errors_have_a_cause() {
        use std::io::ErrorKind as K;
        assert_eq!(io(K::StorageFull).cause(), Cause::DiskFull);
        assert_eq!(io(K::QuotaExceeded).cause(), Cause::DiskFull);
        assert_eq!(io(K::PermissionDenied).cause(), Cause::NoPermission);
        assert_eq!(io(K::NotFound).cause(), Cause::NotFound);
        assert_eq!(io(K::ReadOnlyFilesystem).cause(), Cause::ReadOnly);
        let changed = Error::Io(std::io::Error::other(format!("a.txt {CHANGED_AFTER_SCAN}")));
        assert_eq!(changed.cause(), Cause::FileChanged);
        assert_eq!(io(K::Other).cause(), Cause::Unknown);
        // The headline already says the server could not be reached.
        assert_eq!(
            Error::Rendezvous("connection refused".into()).cause(),
            Cause::Unknown
        );
        assert_eq!(
            Error::Protocol("undecodable message").cause(),
            Cause::Protocol
        );
        assert_eq!(
            Error::Protocol("bad progress report").cause(),
            Cause::Protocol
        );
        assert_eq!(
            Error::Protocol("stream ended early").cause(),
            Cause::ConnectionLost
        );
        assert_eq!(
            Error::Protocol("control stream ended").cause(),
            Cause::ConnectionLost
        );
        assert_eq!(Error::Protocol("send task failed").cause(), Cause::Unknown);
        assert_eq!(Error::Protocol("file too large").cause(), Cause::Unknown);
        // Possibly an attack: no "update" advice.
        let forged = Error::from(totaldrop_crypto::Error::Inauthentic);
        assert_eq!(forged.cause(), Cause::Unknown);
        assert_eq!(Error::WrongCode.cause(), Cause::Unknown);
        assert_eq!(Cause::Unknown.as_str(), "");
        assert_eq!(Cause::TimedOut.as_str(), "TimedOut");
    }
}
