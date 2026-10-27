//! Protocol rules of TotalDrop version 0 that do not involve cryptography:
//! transfer codes (spec section 5), limits and path safety (spec section 11).

pub mod wire;

use unicode_normalization::UnicodeNormalization;

pub mod limits {
    //! Limits from spec section 11. A receiver must reject anything beyond them.

    /// Maximum number of entries in one transfer.
    pub const MAX_ENTRIES: usize = 100_000;
    /// Maximum size of an encoded manifest, in bytes.
    pub const MAX_MANIFEST_BYTES: usize = 16 * 1024 * 1024;
    /// Maximum number of segments in a path.
    pub const MAX_PATH_DEPTH: usize = 64;
    /// Maximum length of one path segment, in bytes of UTF-8.
    pub const MAX_SEGMENT_BYTES: usize = 255;
    /// Maximum length of a whole path joined with `/`, in bytes of UTF-8.
    pub const MAX_PATH_BYTES: usize = 4096;
    /// Plaintext chunk size.
    pub const CHUNK_SIZE: u64 = 1 << 20;
    /// Maximum number of chunks in one file.
    pub const MAX_CHUNKS: u64 = 1 << 32;
}

/// Number of chunks a file of `size` bytes is split into. An empty file is one
/// empty chunk. Returns `None` if the file exceeds [`limits::MAX_CHUNKS`].
pub fn chunk_count(size: u64) -> Option<u64> {
    let n = if size == 0 {
        1
    } else {
        size.div_ceil(limits::CHUNK_SIZE)
    };
    (n <= limits::MAX_CHUNKS).then_some(n)
}

/// Why a path was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
    /// The path has no segments or more than [`limits::MAX_PATH_DEPTH`].
    BadDepth,
    /// The whole path is longer than [`limits::MAX_PATH_BYTES`].
    TooLong,
    /// Segment at this position is empty, `.` or `..`.
    DotOrEmpty(usize),
    /// Segment at this position contains a separator, NUL or control character.
    ForbiddenChar(usize),
    /// Segment at this position contains a bidirectional control character.
    BidiControl(usize),
    /// Segment at this position is a reserved Windows device name.
    ReservedName(usize),
    /// Segment at this position ends in a dot or a space.
    TrailingDotOrSpace(usize),
    /// Segment at this position is longer than [`limits::MAX_SEGMENT_BYTES`].
    SegmentTooLong(usize),
}

/// A path that passed every rule of spec section 11, normalized to NFC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafePath(Vec<String>);

impl SafePath {
    /// The normalized segments.
    pub fn segments(&self) -> &[String] {
        &self.0
    }

    /// The key used to detect paths that differ only in case. Two entries of
    /// one transfer with the same key are a conflict on every platform.
    pub fn collision_key(&self) -> String {
        self.0
            .iter()
            .map(|s| s.to_lowercase())
            .collect::<Vec<_>>()
            .join("/")
    }
}

const RESERVED: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

fn is_bidi_control(c: char) -> bool {
    matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
}

/// Validate and normalize a path received in an `offer` or a container.
pub fn validate_path<S: AsRef<str>>(segments: &[S]) -> Result<SafePath, PathError> {
    if segments.is_empty() || segments.len() > limits::MAX_PATH_DEPTH {
        return Err(PathError::BadDepth);
    }
    let mut out = Vec::with_capacity(segments.len());
    let mut total: usize = 0;
    for (i, raw) in segments.iter().enumerate() {
        let seg: String = raw.as_ref().nfc().collect();
        if seg.is_empty() || seg == "." || seg == ".." {
            return Err(PathError::DotOrEmpty(i));
        }
        if seg
            .chars()
            .any(|c| c == '/' || c == '\\' || c == '\0' || c.is_control())
        {
            return Err(PathError::ForbiddenChar(i));
        }
        if seg.chars().any(is_bidi_control) {
            return Err(PathError::BidiControl(i));
        }
        let stem = seg
            .split('.')
            .next()
            .unwrap_or("")
            .trim_end()
            .to_ascii_uppercase();
        if RESERVED.contains(&stem.as_str()) {
            return Err(PathError::ReservedName(i));
        }
        if seg.ends_with('.') || seg.ends_with(' ') {
            return Err(PathError::TrailingDotOrSpace(i));
        }
        if seg.len() > limits::MAX_SEGMENT_BYTES {
            return Err(PathError::SegmentTooLong(i));
        }
        total = total
            .saturating_add(seg.len())
            .saturating_add(usize::from(i > 0));
        if total > limits::MAX_PATH_BYTES {
            return Err(PathError::TooLong);
        }
        out.push(seg);
    }
    Ok(SafePath(out))
}

/// Why a transfer code could not be parsed or generated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeError {
    /// The code does not start with a valid nameplate number.
    BadNameplate,
    /// The code does not have two or three words.
    WrongWordCount,
    /// This word is not in the list, and is not a valid list index.
    UnknownWord(String),
    /// The operating system's random number generator failed.
    Random,
}

impl core::fmt::Display for CodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CodeError::BadNameplate => {
                f.write_str("the code must start with a number, like 7-harbor-rocket")
            }
            CodeError::WrongWordCount => {
                f.write_str("the code must have two or three words after the number")
            }
            CodeError::UnknownWord(w) => write!(f, "{w:?} is not a code word"),
            CodeError::Random => f.write_str("the random number generator failed"),
        }
    }
}

impl std::error::Error for CodeError {}

/// A transfer code such as `7-harbor-rocket` (spec section 5).
///
/// The protocol works with list indices, never with words: CPace receives
/// [`TransferCode::canonical`], so a code read from a localized word list
/// works for a receiver using the English one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferCode {
    nameplate: u32,
    words: Vec<u16>,
}

fn english() -> &'static [&'static str; 2048] {
    bip39::Language::English.word_list()
}

impl TransferCode {
    /// Pick `word_count` (2 or 3) random words for a nameplate allocated by
    /// the rendezvous server.
    pub fn generate(nameplate: u32, word_count: usize) -> Result<Self, CodeError> {
        if nameplate == 0 {
            return Err(CodeError::BadNameplate);
        }
        if !(2..=3).contains(&word_count) {
            return Err(CodeError::WrongWordCount);
        }
        let mut words = Vec::with_capacity(word_count);
        for _ in 0..word_count {
            // 2^32 is a multiple of 2048, so the low 11 bits are uniform.
            let r = getrandom::u32().map_err(|_| CodeError::Random)?;
            words.push(u16::try_from(r & 0x7FF).unwrap_or(0));
        }
        Ok(Self { nameplate, words })
    }

    /// Parse what a person typed: words or list indices, separated by
    /// hyphens or spaces, in any letter case.
    pub fn parse(input: &str) -> Result<Self, CodeError> {
        let lowered = input.trim().to_lowercase();
        let mut tokens = lowered
            .split(|c: char| c == '-' || c.is_whitespace())
            .filter(|t| !t.is_empty());

        let nameplate = tokens
            .next()
            .filter(|t| t.chars().all(|c| c.is_ascii_digit()))
            .and_then(|t| t.parse::<u32>().ok())
            .filter(|n| *n > 0)
            .ok_or(CodeError::BadNameplate)?;

        let mut words = Vec::new();
        for t in tokens {
            let index = if t.chars().all(|c| c.is_ascii_digit()) {
                t.parse::<u16>().ok().filter(|i| *i < 2048)
            } else {
                bip39::Language::English.find_word(t)
            };
            words.push(index.ok_or_else(|| CodeError::UnknownWord(t.to_string()))?);
        }
        if !(2..=3).contains(&words.len()) {
            return Err(CodeError::WrongWordCount);
        }
        Ok(Self { nameplate, words })
    }

    /// The nameplate number.
    pub fn nameplate(&self) -> u32 {
        self.nameplate
    }

    /// The canonical form given to CPace, for example `7-840-1499`.
    pub fn canonical(&self) -> String {
        let mut s = self.nameplate.to_string();
        for w in &self.words {
            s.push('-');
            s.push_str(&w.to_string());
        }
        s
    }

    /// The form shown to people with the English list, for example `7-harbor-rocket`.
    pub fn to_english(&self) -> String {
        let list = english();
        let mut s = self.nameplate.to_string();
        for w in &self.words {
            s.push('-');
            s.push_str(list.get(usize::from(*w)).copied().unwrap_or("?"));
        }
        s
    }
}

impl core::fmt::Display for TransferCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.to_english())
    }
}

/// The English word at `index` of the 2048-word list, as shown for
/// verification words (spec section 6). `None` for an index of 2048 or more.
pub fn english_word(index: u16) -> Option<&'static str> {
    bip39::Language::English
        .word_list()
        .get(usize::from(index))
        .copied()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn documented_example_code_is_valid() {
        let c = TransferCode::parse("7-harbor-rocket").unwrap();
        assert_eq!(c.canonical(), "7-840-1499");
        assert_eq!(c.to_string(), "7-harbor-rocket");
    }

    #[test]
    fn codes_parse_loosely_but_canonically() {
        let a = TransferCode::parse("  7 Harbor   ROCKET ").unwrap();
        let b = TransferCode::parse("7-840-1499").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn bad_codes_are_rejected() {
        assert_eq!(
            TransferCode::parse("harbor-rocket"),
            Err(CodeError::BadNameplate)
        );
        assert_eq!(
            TransferCode::parse("0-harbor-rocket"),
            Err(CodeError::BadNameplate)
        );
        assert_eq!(
            TransferCode::parse("7-harbor"),
            Err(CodeError::WrongWordCount)
        );
        assert_eq!(
            TransferCode::parse("7-harbor-comet"),
            Err(CodeError::UnknownWord("comet".into()))
        );
        assert_eq!(
            TransferCode::parse("7-harbor-2048"),
            Err(CodeError::UnknownWord("2048".into()))
        );
    }

    #[test]
    fn generated_codes_round_trip() {
        for n in [2, 3] {
            let c = TransferCode::generate(42, n).unwrap();
            assert_eq!(TransferCode::parse(&c.to_string()).unwrap(), c);
            assert_eq!(TransferCode::parse(&c.canonical()).unwrap(), c);
        }
        assert_eq!(
            TransferCode::generate(42, 4),
            Err(CodeError::WrongWordCount)
        );
    }

    #[test]
    fn chunk_counts() {
        assert_eq!(chunk_count(0), Some(1));
        assert_eq!(chunk_count(1), Some(1));
        assert_eq!(chunk_count(limits::CHUNK_SIZE), Some(1));
        assert_eq!(chunk_count(limits::CHUNK_SIZE + 1), Some(2));
        assert_eq!(chunk_count(u64::MAX), None);
    }

    #[test]
    fn safe_paths_pass_and_normalize() {
        let p = validate_path(&["Fotoğraflar", "tatil 2026.jpg"]).unwrap();
        assert_eq!(p.segments().len(), 2);
        // A decomposed "ö" (o + combining diaeresis) becomes the composed form.
        let nfd = validate_path(&["o\u{0308}rnek.txt"]).unwrap();
        assert_eq!(nfd.segments()[0], "\u{00F6}rnek.txt");
    }

    #[test]
    fn traversal_and_tricks_are_rejected() {
        assert_eq!(validate_path(&["..", "etc"]), Err(PathError::DotOrEmpty(0)));
        assert_eq!(validate_path(&["a", ""]), Err(PathError::DotOrEmpty(1)));
        assert_eq!(validate_path(&["a/b"]), Err(PathError::ForbiddenChar(0)));
        assert_eq!(validate_path(&["a\\b"]), Err(PathError::ForbiddenChar(0)));
        assert_eq!(validate_path(&["a\nb"]), Err(PathError::ForbiddenChar(0)));
        assert_eq!(
            validate_path(&["fatura\u{202E}fdp.exe"]),
            Err(PathError::BidiControl(0))
        );
        assert_eq!(validate_path(&["con.txt"]), Err(PathError::ReservedName(0)));
        assert_eq!(validate_path(&["LPT9"]), Err(PathError::ReservedName(0)));
        assert_eq!(
            validate_path(&["notes."]),
            Err(PathError::TrailingDotOrSpace(0))
        );
        assert_eq!(
            validate_path(&["notes "]),
            Err(PathError::TrailingDotOrSpace(0))
        );
        let long = "a".repeat(256);
        assert_eq!(
            validate_path(&[long.as_str()]),
            Err(PathError::SegmentTooLong(0))
        );
        let deep = vec!["a"; 65];
        assert_eq!(validate_path(&deep), Err(PathError::BadDepth));
        let empty: [&str; 0] = [];
        assert_eq!(validate_path(&empty), Err(PathError::BadDepth));
    }

    #[test]
    fn case_only_differences_collide() {
        let a = validate_path(&["Rapor.PDF"]).unwrap();
        let b = validate_path(&["rapor.pdf"]).unwrap();
        assert_eq!(a.collision_key(), b.collision_key());
    }
}
