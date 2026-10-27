//! Wire formats: every protocol message is a CBOR map with small integer
//! keys in deterministic encoding (spec section 2). Receivers ignore unknown
//! keys, reject duplicate keys, and reject trailing bytes.
//!
//! Each message type here has `encode` (always deterministic) and `decode`
//! (strict about what it needs, lenient about what it does not know).

use ciborium::Value;

/// Deepest nesting any v0 message needs (offer → entries → entry → path).
const RECURSION_LIMIT: usize = 16;

/// Why a message could not be decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireError {
    /// Not well-formed CBOR, nested too deeply, or followed by extra bytes.
    Cbor,
    /// The top level is not a map, or a map key appeared twice.
    Shape,
    /// A required field is missing.
    Missing(u64),
    /// A field has the wrong type or an impossible value.
    Field(u64),
    /// The message type (key 0) is not one this side understands here.
    UnknownType(String),
    /// A protocol limit (spec section 11) was exceeded.
    Limit(&'static str),
}

impl core::fmt::Display for WireError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            WireError::Cbor => f.write_str("malformed CBOR"),
            WireError::Shape => f.write_str("message is not a map with unique keys"),
            WireError::Missing(k) => write!(f, "required field {k} is missing"),
            WireError::Field(k) => write!(f, "field {k} has an invalid value"),
            WireError::UnknownType(t) => write!(f, "unexpected message type {t:?}"),
            WireError::Limit(what) => write!(f, "limit exceeded: {what}"),
        }
    }
}

impl std::error::Error for WireError {}

// ---------------------------------------------------------------------------
// Generic helpers

/// A field value being encoded.
fn int(v: u64) -> Value {
    Value::Integer(v.into())
}

fn bytes(v: &[u8]) -> Value {
    Value::Bytes(v.to_vec())
}

fn text(v: &str) -> Value {
    Value::Text(v.to_owned())
}

/// Encode a map in deterministic order. For unsigned integer keys below
/// 2^64, ascending numeric order equals the bytewise order of their
/// shortest encodings, which is what RFC 8949 section 4.2.1 requires.
fn encode_map(mut fields: Vec<(u64, Value)>) -> Vec<u8> {
    fields.sort_by_key(|(k, _)| *k);
    let map = Value::Map(fields.into_iter().map(|(k, v)| (int(k), v)).collect());
    let mut out = Vec::new();
    // Writing to a Vec cannot fail.
    let _ = ciborium::into_writer(&map, &mut out);
    out
}

fn value_to_map(value: Value) -> Result<Fields, WireError> {
    let Value::Map(entries) = value else {
        return Err(WireError::Shape);
    };
    let mut fields: Vec<(u64, Value)> = Vec::with_capacity(entries.len());
    for (k, v) in entries {
        let Value::Integer(i) = k else {
            // Unknown, non-integer keys are ignored like any unknown key.
            continue;
        };
        let Ok(key) = u64::try_from(i) else {
            continue;
        };
        if fields.iter().any(|(existing, _)| *existing == key) {
            return Err(WireError::Shape);
        }
        fields.push((key, v));
    }
    Ok(Fields(fields))
}

fn decode_map(input: &[u8]) -> Result<Fields, WireError> {
    let mut reader = input;
    let value: Value = ciborium::de::from_reader_with_recursion_limit(&mut reader, RECURSION_LIMIT)
        .map_err(|_| WireError::Cbor)?;
    if !reader.is_empty() {
        return Err(WireError::Cbor);
    }
    value_to_map(value)
}

/// The fields of one decoded map.
struct Fields(Vec<(u64, Value)>);

impl Fields {
    fn get(&self, key: u64) -> Option<&Value> {
        self.0.iter().find(|(k, _)| *k == key).map(|(_, v)| v)
    }

    fn take(&mut self, key: u64) -> Option<Value> {
        let pos = self.0.iter().position(|(k, _)| *k == key)?;
        Some(self.0.swap_remove(pos).1)
    }

    fn u64(&self, key: u64) -> Result<u64, WireError> {
        match self.get(key) {
            None => Err(WireError::Missing(key)),
            Some(Value::Integer(i)) => u64::try_from(*i).map_err(|_| WireError::Field(key)),
            Some(_) => Err(WireError::Field(key)),
        }
    }

    fn opt_u64(&self, key: u64) -> Result<Option<u64>, WireError> {
        match self.get(key) {
            None => Ok(None),
            Some(_) => self.u64(key).map(Some),
        }
    }

    fn text(&self, key: u64) -> Result<&str, WireError> {
        match self.get(key) {
            None => Err(WireError::Missing(key)),
            Some(Value::Text(t)) => Ok(t),
            Some(_) => Err(WireError::Field(key)),
        }
    }

    fn opt_text(&self, key: u64) -> Result<Option<&str>, WireError> {
        match self.get(key) {
            None => Ok(None),
            Some(_) => self.text(key).map(Some),
        }
    }

    fn bytes(&self, key: u64) -> Result<&[u8], WireError> {
        match self.get(key) {
            None => Err(WireError::Missing(key)),
            Some(Value::Bytes(b)) => Ok(b),
            Some(_) => Err(WireError::Field(key)),
        }
    }

    fn array<const N: usize>(&self, key: u64) -> Result<[u8; N], WireError> {
        self.bytes(key)?
            .try_into()
            .map_err(|_| WireError::Field(key))
    }

    fn list(&mut self, key: u64) -> Result<Vec<Value>, WireError> {
        match self.take(key) {
            None => Err(WireError::Missing(key)),
            Some(Value::Array(a)) => Ok(a),
            Some(_) => Err(WireError::Field(key)),
        }
    }

    fn kind(&self) -> Result<&str, WireError> {
        self.text(0)
    }
}

fn value_u64(v: &Value, key: u64) -> Result<u64, WireError> {
    match v {
        Value::Integer(i) => u64::try_from(*i).map_err(|_| WireError::Field(key)),
        _ => Err(WireError::Field(key)),
    }
}

// ---------------------------------------------------------------------------
// Rendezvous (spec section 5)

/// How a client leaves a mailbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    /// The transfer was set up.
    Happy,
    /// The other side never came.
    Lonely,
    /// Key confirmation failed: wrong code or an attack.
    Scary,
}

impl Mood {
    fn as_str(self) -> &'static str {
        match self {
            Mood::Happy => "happy",
            Mood::Lonely => "lonely",
            Mood::Scary => "scary",
        }
    }

    fn parse(s: &str, key: u64) -> Result<Self, WireError> {
        match s {
            "happy" => Ok(Mood::Happy),
            "lonely" => Ok(Mood::Lonely),
            "scary" => Ok(Mood::Scary),
            _ => Err(WireError::Field(key)),
        }
    }
}

/// A message from a client to the rendezvous server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientMessage {
    /// `ALLOCATE`: ask for a free nameplate.
    Allocate,
    /// `CLAIM(nameplate)`: join the mailbox.
    Claim {
        /// The nameplate from the code.
        nameplate: u32,
    },
    /// `ADD(phase, body)`: post an opaque message to the other side.
    Add {
        /// Phase name, for example `pake`.
        phase: String,
        /// Opaque body; encrypted for every phase after `pake`.
        body: Vec<u8>,
    },
    /// `CLOSE(mood)`: leave the mailbox.
    Close {
        /// How the session ended.
        mood: Mood,
    },
}

/// Longest phase name accepted.
pub const MAX_PHASE: usize = 32;
/// Largest mailbox message body accepted by the server.
pub const MAX_MAILBOX_BODY: usize = 64 * 1024;

impl ClientMessage {
    /// Deterministic CBOR encoding.
    pub fn encode(&self) -> Vec<u8> {
        match self {
            ClientMessage::Allocate => encode_map(vec![(0, text("allocate"))]),
            ClientMessage::Claim { nameplate } => {
                encode_map(vec![(0, text("claim")), (1, int(u64::from(*nameplate)))])
            }
            ClientMessage::Add { phase, body } => {
                encode_map(vec![(0, text("add")), (1, text(phase)), (2, bytes(body))])
            }
            ClientMessage::Close { mood } => {
                encode_map(vec![(0, text("close")), (1, text(mood.as_str()))])
            }
        }
    }

    /// Strict decoding.
    pub fn decode(input: &[u8]) -> Result<Self, WireError> {
        let f = decode_map(input)?;
        match f.kind()? {
            "allocate" => Ok(ClientMessage::Allocate),
            "claim" => Ok(ClientMessage::Claim {
                nameplate: u32::try_from(f.u64(1)?).map_err(|_| WireError::Field(1))?,
            }),
            "add" => {
                let phase = f.text(1)?;
                if phase.is_empty() || phase.len() > MAX_PHASE {
                    return Err(WireError::Field(1));
                }
                let body = f.bytes(2)?;
                if body.len() > MAX_MAILBOX_BODY {
                    return Err(WireError::Limit("mailbox body"));
                }
                Ok(ClientMessage::Add {
                    phase: phase.to_owned(),
                    body: body.to_vec(),
                })
            }
            "close" => Ok(ClientMessage::Close {
                mood: Mood::parse(f.text(1)?, 1)?,
            }),
            other => Err(WireError::UnknownType(other.to_owned())),
        }
    }
}

/// A message from the rendezvous server to a client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerMessage {
    /// Reply to `ALLOCATE`.
    Allocated {
        /// The nameplate reserved for this client.
        nameplate: u32,
    },
    /// Reply to `CLAIM`.
    Claimed {
        /// Random 16-byte mailbox ID, the CPace session ID.
        mailbox: [u8; 16],
    },
    /// A message the other side posted with `ADD`.
    Message {
        /// Its phase.
        phase: String,
        /// Its body.
        body: Vec<u8>,
    },
    /// The other side closed the mailbox.
    PeerClosed {
        /// Its mood.
        mood: Mood,
    },
    /// The request was refused. The connection is closed afterwards.
    Error {
        /// A short reason, for example `nameplate taken`.
        reason: String,
    },
}

impl ServerMessage {
    /// Deterministic CBOR encoding.
    pub fn encode(&self) -> Vec<u8> {
        match self {
            ServerMessage::Allocated { nameplate } => encode_map(vec![
                (0, text("allocated")),
                (1, int(u64::from(*nameplate))),
            ]),
            ServerMessage::Claimed { mailbox } => {
                encode_map(vec![(0, text("claimed")), (1, bytes(mailbox))])
            }
            ServerMessage::Message { phase, body } => encode_map(vec![
                (0, text("message")),
                (1, text(phase)),
                (2, bytes(body)),
            ]),
            ServerMessage::PeerClosed { mood } => {
                encode_map(vec![(0, text("peer_closed")), (1, text(mood.as_str()))])
            }
            ServerMessage::Error { reason } => {
                encode_map(vec![(0, text("error")), (1, text(reason))])
            }
        }
    }

    /// Strict decoding.
    pub fn decode(input: &[u8]) -> Result<Self, WireError> {
        let f = decode_map(input)?;
        match f.kind()? {
            "allocated" => Ok(ServerMessage::Allocated {
                nameplate: u32::try_from(f.u64(1)?).map_err(|_| WireError::Field(1))?,
            }),
            "claimed" => Ok(ServerMessage::Claimed {
                mailbox: f.array(1)?,
            }),
            "message" => {
                let body = f.bytes(2)?;
                if body.len() > MAX_MAILBOX_BODY {
                    return Err(WireError::Limit("mailbox body"));
                }
                Ok(ServerMessage::Message {
                    phase: f.text(1)?.to_owned(),
                    body: body.to_vec(),
                })
            }
            "peer_closed" => Ok(ServerMessage::PeerClosed {
                mood: Mood::parse(f.text(1)?, 1)?,
            }),
            "error" => Ok(ServerMessage::Error {
                reason: f.text(1)?.chars().take(200).collect(),
            }),
            other => Err(WireError::UnknownType(other.to_owned())),
        }
    }
}

// ---------------------------------------------------------------------------
// Device certificate (spec section 4)

/// Size of the ML-KEM-768 encapsulation key in a certificate.
pub const KEM_KEY_SIZE: usize = 1184;

/// A device certificate: the device's three public keys, signed by its
/// Ed25519 key. Verification lives with the caller, which has the crypto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Certificate {
    /// Field 1, always 0 in v0.
    pub version: u64,
    /// Field 2, Ed25519 public key.
    pub signing_key: [u8; 32],
    /// Field 3, X25519 public key (Noise static key).
    pub static_key: [u8; 32],
    /// Field 4, ML-KEM-768 encapsulation key.
    pub kem_key: Vec<u8>,
    /// Field 5, seconds since the Unix epoch.
    pub created: u64,
    /// Field 6, Ed25519 signature over [`Certificate::signed_bytes`].
    pub signature: [u8; 64],
}

impl Certificate {
    fn body_fields(&self) -> Vec<(u64, Value)> {
        vec![
            (1, int(self.version)),
            (2, bytes(&self.signing_key)),
            (3, bytes(&self.static_key)),
            (4, bytes(&self.kem_key)),
            (5, int(self.created)),
        ]
    }

    /// The bytes the signature covers: the deterministic encoding of
    /// fields 1–5. Recomputed from the parsed fields, never taken from the
    /// wire, so a different encoding of the same values cannot slip through.
    pub fn signed_bytes(&self) -> Vec<u8> {
        encode_map(self.body_fields())
    }

    /// Deterministic CBOR encoding of all six fields.
    pub fn encode(&self) -> Vec<u8> {
        let mut fields = self.body_fields();
        fields.push((6, bytes(&self.signature)));
        encode_map(fields)
    }

    /// Strict decoding. Does not check the signature.
    pub fn decode(input: &[u8]) -> Result<Self, WireError> {
        let f = decode_map(input)?;
        let version = f.u64(1)?;
        if version != 0 {
            return Err(WireError::Field(1));
        }
        let kem_key = f.bytes(4)?;
        if kem_key.len() != KEM_KEY_SIZE {
            return Err(WireError::Field(4));
        }
        Ok(Certificate {
            version,
            signing_key: f.array(2)?,
            static_key: f.array(3)?,
            kem_key: kem_key.to_vec(),
            created: f.u64(5)?,
            signature: f.array(6)?,
        })
    }
}

// ---------------------------------------------------------------------------
// HINTS (spec section 6)

/// An IP address and UDP port where a device may be reachable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Address(pub std::net::SocketAddr);

fn encode_addr(a: &Address) -> Value {
    let ip = match a.0.ip() {
        std::net::IpAddr::V4(v4) => v4.octets().to_vec(),
        std::net::IpAddr::V6(v6) => v6.octets().to_vec(),
    };
    Value::Array(vec![Value::Bytes(ip), int(u64::from(a.0.port()))])
}

fn decode_addr(v: &Value, key: u64) -> Result<Address, WireError> {
    let Value::Array(parts) = v else {
        return Err(WireError::Field(key));
    };
    let [Value::Bytes(ip), port] = parts.as_slice() else {
        return Err(WireError::Field(key));
    };
    let port = u16::try_from(value_u64(port, key)?).map_err(|_| WireError::Field(key))?;
    let ip = match ip.len() {
        4 => {
            let o: [u8; 4] = ip
                .as_slice()
                .try_into()
                .map_err(|_| WireError::Field(key))?;
            std::net::IpAddr::from(o)
        }
        16 => {
            let o: [u8; 16] = ip
                .as_slice()
                .try_into()
                .map_err(|_| WireError::Field(key))?;
            std::net::IpAddr::from(o)
        }
        _ => return Err(WireError::Field(key)),
    };
    Ok(Address(std::net::SocketAddr::new(ip, port)))
}

/// Most addresses accepted in one list, so a peer cannot make us spray
/// packets at many hosts.
pub const MAX_HINT_ADDRESSES: usize = 8;

/// Connection hints, sent encrypted under `K_mailbox` after key confirmation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hints {
    /// Field 1: protocol versions this side supports.
    pub versions: Vec<u64>,
    /// Field 2: the single-use Ed25519 endpoint key for this transfer.
    pub endpoint_key: [u8; 32],
    /// Field 3: public addresses observed via the relay.
    pub public: Vec<Address>,
    /// Field 4: local addresses.
    pub local: Vec<Address>,
    /// Field 5: home relay URL, if any.
    pub relay: Option<String>,
}

impl Hints {
    /// Deterministic CBOR encoding.
    pub fn encode(&self) -> Vec<u8> {
        let mut fields = vec![
            (
                1,
                Value::Array(self.versions.iter().map(|v| int(*v)).collect()),
            ),
            (2, bytes(&self.endpoint_key)),
            (
                3,
                Value::Array(self.public.iter().map(encode_addr).collect()),
            ),
            (
                4,
                Value::Array(self.local.iter().map(encode_addr).collect()),
            ),
        ];
        if let Some(r) = &self.relay {
            fields.push((5, text(r)));
        }
        encode_map(fields)
    }

    /// Strict decoding.
    pub fn decode(input: &[u8]) -> Result<Self, WireError> {
        let mut f = decode_map(input)?;
        let versions = f
            .list(1)?
            .iter()
            .map(|v| value_u64(v, 1))
            .collect::<Result<Vec<_>, _>>()?;
        if versions.is_empty() || versions.len() > 16 {
            return Err(WireError::Field(1));
        }
        let mut addrs = |key: u64| -> Result<Vec<Address>, WireError> {
            let list = f.list(key)?;
            if list.len() > MAX_HINT_ADDRESSES {
                return Err(WireError::Limit("hint addresses"));
            }
            list.iter().map(|v| decode_addr(v, key)).collect()
        };
        let public = addrs(3)?;
        let local = addrs(4)?;
        Ok(Hints {
            versions,
            endpoint_key: f.array(2)?,
            public,
            local,
            relay: f.opt_text(5)?.map(str::to_owned),
        })
    }
}

// ---------------------------------------------------------------------------
// Transfer protocol (spec section 8)

/// Kind of an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    /// A regular file.
    File,
    /// A directory.
    Dir,
}

/// One file or directory of a transfer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Field 1: path segments, validated by the receiver before use.
    pub path: Vec<String>,
    /// Field 2.
    pub kind: EntryKind,
    /// Field 3: size in bytes; 0 for directories.
    pub size: u64,
    /// Field 4: BLAKE3 hash of the plaintext; required for files.
    pub hash: Option<[u8; 32]>,
    /// Field 5: modification time, seconds since the Unix epoch.
    pub mtime: Option<u64>,
}

impl Entry {
    fn to_value(&self) -> Value {
        let mut fields = vec![
            (1, Value::Array(self.path.iter().map(|s| text(s)).collect())),
            (
                2,
                text(match self.kind {
                    EntryKind::File => "file",
                    EntryKind::Dir => "dir",
                }),
            ),
            (3, int(self.size)),
        ];
        if let Some(h) = &self.hash {
            fields.push((4, bytes(h)));
        }
        if let Some(m) = self.mtime {
            fields.push((5, int(m)));
        }
        fields.sort_by_key(|(k, _)| *k);
        Value::Map(fields.into_iter().map(|(k, v)| (int(k), v)).collect())
    }

    fn from_value(v: Value) -> Result<Self, WireError> {
        let mut f = value_to_map(v)?;
        let path = f
            .list(1)?
            .into_iter()
            .map(|s| match s {
                Value::Text(t) => Ok(t),
                _ => Err(WireError::Field(1)),
            })
            .collect::<Result<Vec<_>, _>>()?;
        if path.len() > crate::limits::MAX_PATH_DEPTH {
            return Err(WireError::Limit("path depth"));
        }
        let kind = match f.text(2)? {
            "file" => EntryKind::File,
            "dir" => EntryKind::Dir,
            _ => return Err(WireError::Field(2)),
        };
        let size = f.u64(3)?;
        let hash = match f.get(4) {
            None => None,
            Some(_) => Some(f.array(4)?),
        };
        match kind {
            EntryKind::File if hash.is_none() => return Err(WireError::Missing(4)),
            EntryKind::Dir if size != 0 => return Err(WireError::Field(3)),
            _ => {}
        }
        if crate::chunk_count(size).is_none() {
            return Err(WireError::Limit("chunks per file"));
        }
        Ok(Entry {
            path,
            kind,
            size,
            hash,
            mtime: f.opt_u64(5)?,
        })
    }
}

/// Why a receiver turned an offer down.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectReason {
    /// The person said no.
    Declined,
    /// Bigger than the receiver accepts.
    TooLarge,
    /// Not enough disk space.
    NoSpace,
    /// The offer broke a rule (bad path, bad signature, over a limit).
    Invalid,
}

impl RejectReason {
    fn as_str(self) -> &'static str {
        match self {
            RejectReason::Declined => "declined",
            RejectReason::TooLarge => "too_large",
            RejectReason::NoSpace => "no_space",
            RejectReason::Invalid => "invalid",
        }
    }

    fn parse(s: &str) -> Self {
        match s {
            "declined" => RejectReason::Declined,
            "too_large" => RejectReason::TooLarge,
            "no_space" => RejectReason::NoSpace,
            // Unknown reasons from newer versions read as "invalid".
            _ => RejectReason::Invalid,
        }
    }
}

/// The signed part of an offer (fields 1–4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// Field 1: random transfer ID.
    pub transfer_id: [u8; 16],
    /// Field 2: entries.
    pub entries: Vec<Entry>,
    /// Field 3: total size of all files.
    pub total_size: u64,
    /// Field 4: chunk size, always 1 MiB in v0.
    pub chunk_size: u64,
}

impl Manifest {
    fn fields(&self) -> Vec<(u64, Value)> {
        vec![
            (1, bytes(&self.transfer_id)),
            (
                2,
                Value::Array(self.entries.iter().map(Entry::to_value).collect()),
            ),
            (3, int(self.total_size)),
            (4, int(self.chunk_size)),
        ]
    }

    /// Deterministic encoding of fields 1–4: what the sender signs.
    pub fn signed_bytes(&self) -> Vec<u8> {
        encode_map(self.fields())
    }
}

/// A message on the control stream (spec section 8), plus the key
/// confirmation message of section 6.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Control {
    /// Key confirmation under the hybrid session keys.
    Confirm,
    /// `offer`, sender to receiver.
    Offer {
        /// Fields 1–4.
        manifest: Manifest,
        /// Field 5: content key `CK`.
        content_key: [u8; 32],
        /// Field 6: Ed25519 signature by the sender's device key over fields 1–4.
        signature: [u8; 64],
    },
    /// `accept`, receiver to sender.
    Accept {
        /// Field 1.
        transfer_id: [u8; 16],
        /// Field 2: wanted entries, each with a bitmap of chunks already held
        /// (bit `n` of byte `n / 8`, least significant bit first).
        wanted: Vec<(u64, Vec<u8>)>,
    },
    /// `reject`, receiver to sender.
    Reject {
        /// Field 1.
        transfer_id: [u8; 16],
        /// Field 2.
        reason: RejectReason,
    },
    /// `entry_done`, receiver to sender.
    EntryDone {
        /// Field 1.
        transfer_id: [u8; 16],
        /// Field 2.
        index: u64,
        /// Field 3: true for `ok`, false for `hash_mismatch`.
        ok: bool,
    },
    /// `cancel`, either side.
    Cancel {
        /// Field 1.
        transfer_id: [u8; 16],
        /// Field 2.
        reason: String,
    },
}

impl Control {
    /// Deterministic CBOR encoding.
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Control::Confirm => encode_map(vec![(0, text("confirm"))]),
            Control::Offer {
                manifest,
                content_key,
                signature,
            } => {
                let mut fields = manifest.fields();
                fields.push((0, text("offer")));
                fields.push((5, bytes(content_key)));
                fields.push((6, bytes(signature)));
                encode_map(fields)
            }
            Control::Accept {
                transfer_id,
                wanted,
            } => encode_map(vec![
                (0, text("accept")),
                (1, bytes(transfer_id)),
                (
                    2,
                    Value::Array(
                        wanted
                            .iter()
                            .map(|(i, bm)| Value::Array(vec![int(*i), bytes(bm)]))
                            .collect(),
                    ),
                ),
            ]),
            Control::Reject {
                transfer_id,
                reason,
            } => encode_map(vec![
                (0, text("reject")),
                (1, bytes(transfer_id)),
                (2, text(reason.as_str())),
            ]),
            Control::EntryDone {
                transfer_id,
                index,
                ok,
            } => encode_map(vec![
                (0, text("entry_done")),
                (1, bytes(transfer_id)),
                (2, int(*index)),
                (3, text(if *ok { "ok" } else { "hash_mismatch" })),
            ]),
            Control::Cancel {
                transfer_id,
                reason,
            } => encode_map(vec![
                (0, text("cancel")),
                (1, bytes(transfer_id)),
                (2, text(reason)),
            ]),
        }
    }

    /// Strict decoding, enforcing the limits of spec section 11.
    pub fn decode(input: &[u8]) -> Result<Self, WireError> {
        if input.len() > crate::limits::MAX_MANIFEST_BYTES {
            return Err(WireError::Limit("manifest size"));
        }
        let mut f = decode_map(input)?;
        let kind = f.kind()?.to_owned();
        match kind.as_str() {
            "confirm" => Ok(Control::Confirm),
            "offer" => {
                let list = f.list(2)?;
                if list.len() > crate::limits::MAX_ENTRIES {
                    return Err(WireError::Limit("entries per transfer"));
                }
                let entries = list
                    .into_iter()
                    .map(Entry::from_value)
                    .collect::<Result<Vec<_>, _>>()?;
                let chunk_size = f.u64(4)?;
                if chunk_size != crate::limits::CHUNK_SIZE {
                    return Err(WireError::Field(4));
                }
                let total_size = f.u64(3)?;
                let sum = entries
                    .iter()
                    .try_fold(0u64, |acc, e| acc.checked_add(e.size))
                    .ok_or(WireError::Field(3))?;
                if sum != total_size {
                    return Err(WireError::Field(3));
                }
                Ok(Control::Offer {
                    manifest: Manifest {
                        transfer_id: f.array(1)?,
                        entries,
                        total_size,
                        chunk_size,
                    },
                    content_key: f.array(5)?,
                    signature: f.array(6)?,
                })
            }
            "accept" => {
                let list = f.list(2)?;
                if list.len() > crate::limits::MAX_ENTRIES {
                    return Err(WireError::Limit("entries per transfer"));
                }
                let wanted = list
                    .into_iter()
                    .map(|v| match v {
                        Value::Array(pair) => match pair.as_slice() {
                            [i, Value::Bytes(bm)] => Ok((value_u64(i, 2)?, bm.clone())),
                            _ => Err(WireError::Field(2)),
                        },
                        _ => Err(WireError::Field(2)),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Control::Accept {
                    transfer_id: f.array(1)?,
                    wanted,
                })
            }
            "reject" => Ok(Control::Reject {
                transfer_id: f.array(1)?,
                reason: RejectReason::parse(f.text(2)?),
            }),
            "entry_done" => Ok(Control::EntryDone {
                transfer_id: f.array(1)?,
                index: f.u64(2)?,
                ok: match f.text(3)? {
                    "ok" => true,
                    "hash_mismatch" => false,
                    _ => return Err(WireError::Field(3)),
                },
            }),
            "cancel" => Ok(Control::Cancel {
                transfer_id: f.array(1)?,
                reason: f.text(2)?.chars().take(200).collect(),
            }),
            other => Err(WireError::UnknownType(other.to_owned())),
        }
    }
}

/// The header that opens every data stream (spec section 8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataHeader {
    /// Field 1.
    pub transfer_id: [u8; 16],
    /// Field 2.
    pub entry: u64,
    /// Field 3.
    pub first_chunk: u64,
}

impl DataHeader {
    /// Deterministic CBOR encoding.
    pub fn encode(&self) -> Vec<u8> {
        encode_map(vec![
            (1, bytes(&self.transfer_id)),
            (2, int(self.entry)),
            (3, int(self.first_chunk)),
        ])
    }

    /// Strict decoding.
    pub fn decode(input: &[u8]) -> Result<Self, WireError> {
        let f = decode_map(input)?;
        Ok(DataHeader {
            transfer_id: f.array(1)?,
            entry: f.u64(2)?,
            first_chunk: f.u64(3)?,
        })
    }
}

// ---------------------------------------------------------------------------
// Framing (spec section 2)

/// Largest framed message: 1 MiB plus 1 KiB of headroom.
pub const MAX_FRAME: usize = (1 << 20) + 1024;

/// Unsigned LEB128 encoding of a frame length.
pub fn encode_length(mut n: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(4);
    loop {
        let low = u8::try_from(n & 0x7f).unwrap_or(0);
        n >>= 7;
        if n == 0 {
            out.push(low);
            return out;
        }
        out.push(low | 0x80);
    }
}

/// Incremental LEB128 length decoder, fed one byte at a time.
/// Rejects lengths above `max` and non-minimal encodings.
#[derive(Debug, Clone)]
pub struct LengthDecoder {
    value: u64,
    shift: u32,
    max: usize,
}

impl LengthDecoder {
    /// A decoder for frames of at most `max` bytes.
    pub fn new(max: usize) -> Self {
        Self {
            value: 0,
            shift: 0,
            max,
        }
    }

    /// Feed the next byte. Returns `Ok(Some(len))` when complete.
    pub fn push(&mut self, byte: u8) -> Result<Option<usize>, WireError> {
        if self.shift >= 35 {
            return Err(WireError::Limit("frame length"));
        }
        let part = u64::from(byte & 0x7f);
        if self.shift > 0 && byte == 0 {
            // A trailing zero byte would be a non-minimal encoding.
            return Err(WireError::Cbor);
        }
        self.value |= part.checked_shl(self.shift).ok_or(WireError::Cbor)?;
        self.shift = self.shift.saturating_add(7);
        if byte & 0x80 != 0 {
            return Ok(None);
        }
        let len = usize::try_from(self.value).map_err(|_| WireError::Limit("frame length"))?;
        if len > self.max {
            return Err(WireError::Limit("frame length"));
        }
        Ok(Some(len))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn rendezvous_round_trip() {
        let msgs = [
            ClientMessage::Allocate,
            ClientMessage::Claim { nameplate: 7 },
            ClientMessage::Add {
                phase: "pake".into(),
                body: vec![1, 2, 3],
            },
            ClientMessage::Close { mood: Mood::Scary },
        ];
        for m in msgs {
            assert_eq!(ClientMessage::decode(&m.encode()).unwrap(), m);
        }
        let msgs = [
            ServerMessage::Allocated { nameplate: 1 },
            ServerMessage::Claimed { mailbox: [9; 16] },
            ServerMessage::Message {
                phase: "hints".into(),
                body: vec![0; 40],
            },
            ServerMessage::PeerClosed { mood: Mood::Happy },
            ServerMessage::Error {
                reason: "nameplate taken".into(),
            },
        ];
        for m in msgs {
            assert_eq!(ServerMessage::decode(&m.encode()).unwrap(), m);
        }
    }

    #[test]
    fn unknown_keys_are_ignored_duplicates_and_trailing_bytes_are_not() {
        // {0: "claim", 1: 7, 99: "future"}
        let with_extra = encode_map(vec![(0, text("claim")), (1, int(7)), (99, text("future"))]);
        assert_eq!(
            ClientMessage::decode(&with_extra).unwrap(),
            ClientMessage::Claim { nameplate: 7 }
        );
        // Duplicate key 1, written by hand: a3 00 65 "claim" 01 07 01 08
        let dup = [
            0xa3, 0x00, 0x65, b'c', b'l', b'a', b'i', b'm', 0x01, 0x07, 0x01, 0x08,
        ];
        assert_eq!(ClientMessage::decode(&dup).unwrap_err(), WireError::Shape);
        let mut trailing = ClientMessage::Allocate.encode();
        trailing.push(0);
        assert_eq!(
            ClientMessage::decode(&trailing).unwrap_err(),
            WireError::Cbor
        );
    }

    #[test]
    fn encoding_is_deterministic_and_sorted() {
        // Keys given out of order come out sorted: {0: "allocate"} is a1 00 68 ...
        let a = ClientMessage::Allocate.encode();
        assert_eq!(&a[..2], &[0xa1, 0x00]);
        let h = DataHeader {
            transfer_id: [1; 16],
            entry: 2,
            first_chunk: 3,
        };
        let e = h.encode();
        assert_eq!(e[0], 0xa3);
        assert_eq!(e[1], 0x01);
        assert_eq!(DataHeader::decode(&e).unwrap(), h);
    }

    fn sample_offer() -> Control {
        Control::Offer {
            manifest: Manifest {
                transfer_id: [4; 16],
                entries: vec![
                    Entry {
                        path: vec!["docs".into()],
                        kind: EntryKind::Dir,
                        size: 0,
                        hash: None,
                        mtime: None,
                    },
                    Entry {
                        path: vec!["docs".into(), "a.txt".into()],
                        kind: EntryKind::File,
                        size: 5,
                        hash: Some([7; 32]),
                        mtime: Some(1_700_000_000),
                    },
                ],
                total_size: 5,
                chunk_size: 1 << 20,
            },
            content_key: [8; 32],
            signature: [9; 64],
        }
    }

    #[test]
    fn control_round_trip() {
        let msgs = [
            Control::Confirm,
            sample_offer(),
            Control::Accept {
                transfer_id: [1; 16],
                wanted: vec![(0, vec![]), (1, vec![0b101])],
            },
            Control::Reject {
                transfer_id: [1; 16],
                reason: RejectReason::NoSpace,
            },
            Control::EntryDone {
                transfer_id: [1; 16],
                index: 3,
                ok: false,
            },
            Control::Cancel {
                transfer_id: [1; 16],
                reason: "stopped".into(),
            },
        ];
        for m in msgs {
            assert_eq!(Control::decode(&m.encode()).unwrap(), m);
        }
    }

    #[test]
    fn offer_rules() {
        let Control::Offer {
            mut manifest,
            content_key,
            signature,
        } = sample_offer()
        else {
            unreachable!()
        };
        // Total size must match the sum of entries.
        manifest.total_size = 6;
        let bad = Control::Offer {
            manifest: manifest.clone(),
            content_key,
            signature,
        };
        assert_eq!(
            Control::decode(&bad.encode()).unwrap_err(),
            WireError::Field(3)
        );
        // Chunk size is fixed.
        manifest.total_size = 5;
        manifest.chunk_size = 4096;
        let bad = Control::Offer {
            manifest: manifest.clone(),
            content_key,
            signature,
        };
        assert_eq!(
            Control::decode(&bad.encode()).unwrap_err(),
            WireError::Field(4)
        );
        // Files need a hash.
        manifest.chunk_size = 1 << 20;
        manifest.entries[1].hash = None;
        let bad = Control::Offer {
            manifest,
            content_key,
            signature,
        };
        assert_eq!(
            Control::decode(&bad.encode()).unwrap_err(),
            WireError::Missing(4)
        );
    }

    #[test]
    fn signed_bytes_exclude_the_content_key() {
        let Control::Offer { manifest, .. } = sample_offer() else {
            unreachable!()
        };
        let signed = manifest.signed_bytes();
        assert!(!signed.windows(32).any(|w| w == [8u8; 32]));
    }

    #[test]
    fn hints_round_trip_and_limits() {
        let h = Hints {
            versions: vec![0],
            endpoint_key: [3; 32],
            public: vec![],
            local: vec![
                Address("127.0.0.1:4000".parse().unwrap()),
                Address("[::1]:5000".parse().unwrap()),
            ],
            relay: None,
        };
        assert_eq!(Hints::decode(&h.encode()).unwrap(), h);
        let many = Hints {
            local: vec![Address("10.0.0.1:1".parse().unwrap()); MAX_HINT_ADDRESSES + 1],
            ..h
        };
        assert_eq!(
            Hints::decode(&many.encode()).unwrap_err(),
            WireError::Limit("hint addresses")
        );
    }

    #[test]
    fn certificate_round_trip_and_signed_bytes() {
        let c = Certificate {
            version: 0,
            signing_key: [1; 32],
            static_key: [2; 32],
            kem_key: vec![3; KEM_KEY_SIZE],
            created: 1_800_000_000,
            signature: [4; 64],
        };
        let enc = c.encode();
        assert_eq!(Certificate::decode(&enc).unwrap(), c);
        assert!(!c.signed_bytes().windows(64).any(|w| w == [4u8; 64]));
        let bad = Certificate {
            kem_key: vec![3; 10],
            ..c
        };
        assert_eq!(
            Certificate::decode(&bad.encode()).unwrap_err(),
            WireError::Field(4)
        );
    }

    #[test]
    fn framing() {
        for n in [0usize, 1, 127, 128, 300, MAX_FRAME] {
            let enc = encode_length(n);
            let mut d = LengthDecoder::new(MAX_FRAME);
            let mut got = None;
            for b in enc {
                got = d.push(b).unwrap();
            }
            assert_eq!(got, Some(n));
        }
        let mut d = LengthDecoder::new(MAX_FRAME);
        let enc = encode_length(MAX_FRAME + 1);
        let mut res = Ok(None);
        for b in enc {
            res = d.push(b);
            if res.is_err() {
                break;
            }
        }
        assert!(res.is_err());
        // Non-minimal: 0x80 0x00 encodes 0 in two bytes.
        let mut d = LengthDecoder::new(MAX_FRAME);
        assert_eq!(d.push(0x80).unwrap(), None);
        assert!(d.push(0x00).is_err());
        // Endless continuation bytes are cut off.
        let mut d = LengthDecoder::new(usize::MAX);
        let mut failed = false;
        for _ in 0..10 {
            if d.push(0xff).is_err() {
                failed = true;
                break;
            }
        }
        assert!(failed);
    }

    /// A cheap, always-on stand-in for the fuzzers: every single-byte change
    /// and every truncation of valid messages must decode or fail cleanly,
    /// and whatever decodes must round-trip.
    #[test]
    fn mutations_never_panic_and_round_trip() {
        let samples = vec![
            sample_offer().encode(),
            ClientMessage::Add {
                phase: "hints".into(),
                body: vec![1; 20],
            }
            .encode(),
            Hints {
                versions: vec![0],
                endpoint_key: [3; 32],
                public: vec![Address("1.2.3.4:5".parse().unwrap())],
                local: vec![],
                relay: Some("wss://r".into()),
            }
            .encode(),
        ];
        for sample in samples {
            for cut in 0..sample.len() {
                check_all(&sample[..cut]);
            }
            for pos in 0..sample.len() {
                for delta in [1u8, 0x80, 0xff] {
                    let mut m = sample.clone();
                    m[pos] = m[pos].wrapping_add(delta);
                    check_all(&m);
                }
            }
        }
    }

    fn check_all(data: &[u8]) {
        if let Ok(m) = ClientMessage::decode(data) {
            assert_eq!(ClientMessage::decode(&m.encode()).unwrap(), m);
        }
        if let Ok(m) = ServerMessage::decode(data) {
            assert_eq!(ServerMessage::decode(&m.encode()).unwrap(), m);
        }
        if let Ok(m) = Control::decode(data) {
            assert_eq!(Control::decode(&m.encode()).unwrap(), m);
        }
        if let Ok(m) = Hints::decode(data) {
            assert_eq!(Hints::decode(&m.encode()).unwrap(), m);
        }
        if let Ok(m) = DataHeader::decode(data) {
            assert_eq!(DataHeader::decode(&m.encode()).unwrap(), m);
        }
    }
}
