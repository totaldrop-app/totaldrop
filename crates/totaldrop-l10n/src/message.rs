//! A small, strict parser and formatter for the subset of ICU MessageFormat
//! that TotalDrop's ARB files use: `{name}`, `{n, plural, =1{…} one{…}
//! other{…}}` and `{x, select, a{…} other{…}}`.
//!
//! Apostrophes are plain text, as in Flutter's gen_l10n without
//! `use-escaping`. There is no `#` shorthand: the count is written as
//! `{count}` inside the case.
//!
//! Translations come from outside contributors, so parsing is bounded in
//! length and depth and never panics.

use std::collections::BTreeSet;

/// The longest message accepted, in bytes.
pub const MAX_LEN: usize = 16 * 1024;
/// How deeply plurals and selects may nest.
pub const MAX_DEPTH: usize = 4;

/// A parsed message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message(Vec<Part>);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Part {
    Text(String),
    Arg(String),
    Plural {
        arg: String,
        cases: Vec<(Key, Message)>,
    },
    Select {
        arg: String,
        cases: Vec<(String, Message)>,
    },
}

/// A plural case: an exact number or a CLDR category name.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Key {
    Exact(u64),
    Category(String),
}

/// Why a message could not be parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Longer than [`MAX_LEN`].
    TooLong,
    /// Nested deeper than [`MAX_DEPTH`].
    TooDeep,
    /// Bad syntax at this byte offset.
    Syntax(usize),
    /// A plural or select without an `other` case.
    NoOther(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooLong => write!(f, "message longer than {MAX_LEN} bytes"),
            Self::TooDeep => write!(f, "nested deeper than {MAX_DEPTH}"),
            Self::Syntax(at) => write!(f, "syntax error at byte {at}"),
            Self::NoOther(arg) => write!(f, "`{arg}` has no `other` case"),
        }
    }
}

impl std::error::Error for ParseError {}

/// A value for an argument.
#[derive(Debug, Clone, Copy)]
pub enum Arg<'a> {
    /// Text, inserted as it is.
    Text(&'a str),
    /// A count: chooses plural cases and is inserted as a localized number.
    Count(u64),
}

/// What formatting needs from the locale.
pub trait Locale {
    /// The CLDR plural category (`zero`, `one`, `two`, `few`, `many`,
    /// `other`) of `n`.
    fn plural(&self, n: u64) -> &'static str;
    /// `n` with the locale's digits and grouping.
    fn number(&self, n: u64) -> String;
    /// Text inserted into a sentence, for example isolated from the
    /// surrounding direction in a right-to-left language.
    fn text(&self, s: &str, out: &mut String);
}

struct Parser<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<char> {
        self.src.get(self.pos..).and_then(|s| s.chars().next())
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos = self.pos.saturating_add(c.len_utf8());
        Some(c)
    }

    fn eat(&mut self, want: char) -> Result<(), ParseError> {
        if self.peek() == Some(want) {
            self.bump();
            Ok(())
        } else {
            Err(ParseError::Syntax(self.pos))
        }
    }

    fn spaces(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.bump();
        }
    }

    fn ident(&mut self) -> Result<&'a str, ParseError> {
        let start = self.pos;
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            self.bump();
        }
        let s = self.src.get(start..self.pos).unwrap_or_default();
        if s.is_empty() || s.starts_with(|c: char| c.is_ascii_digit()) {
            return Err(ParseError::Syntax(start));
        }
        Ok(s)
    }

    /// Parts until the end of input (top level) or an unmatched `}`.
    fn message(&mut self, depth: usize) -> Result<Message, ParseError> {
        if depth > MAX_DEPTH {
            return Err(ParseError::TooDeep);
        }
        let mut parts = Vec::new();
        let mut text = String::new();
        while let Some(c) = self.peek() {
            match c {
                '{' => {
                    if !text.is_empty() {
                        parts.push(Part::Text(std::mem::take(&mut text)));
                    }
                    self.bump();
                    parts.push(self.placeholder(depth)?);
                }
                '}' => break,
                _ => {
                    text.push(c);
                    self.bump();
                }
            }
        }
        if !text.is_empty() {
            parts.push(Part::Text(text));
        }
        Ok(Message(parts))
    }

    /// After `{`: `name}`, `name, plural, cases}` or `name, select, cases}`.
    fn placeholder(&mut self, depth: usize) -> Result<Part, ParseError> {
        self.spaces();
        let arg = self.ident()?.to_owned();
        self.spaces();
        if self.peek() == Some('}') {
            self.bump();
            return Ok(Part::Arg(arg));
        }
        self.eat(',')?;
        self.spaces();
        let kind_at = self.pos;
        let kind = self.ident()?;
        self.spaces();
        self.eat(',')?;
        match kind {
            "plural" => {
                let mut cases = Vec::new();
                for (key, body) in self.cases(depth)? {
                    let key = match key.strip_prefix('=') {
                        Some(n) => Key::Exact(n.parse().map_err(|_| ParseError::Syntax(kind_at))?),
                        None => match key.as_str() {
                            "zero" | "one" | "two" | "few" | "many" | "other" => Key::Category(key),
                            _ => return Err(ParseError::Syntax(kind_at)),
                        },
                    };
                    cases.push((key, body));
                }
                if !cases
                    .iter()
                    .any(|(k, _)| *k == Key::Category("other".into()))
                {
                    return Err(ParseError::NoOther(arg));
                }
                Ok(Part::Plural { arg, cases })
            }
            "select" => {
                let cases = self.cases(depth)?;
                if !cases.iter().any(|(k, _)| k == "other") {
                    return Err(ParseError::NoOther(arg));
                }
                Ok(Part::Select { arg, cases })
            }
            _ => Err(ParseError::Syntax(kind_at)),
        }
    }

    /// `key{body} key{body} … }`
    fn cases(&mut self, depth: usize) -> Result<Vec<(String, Message)>, ParseError> {
        let mut cases = Vec::new();
        loop {
            self.spaces();
            match self.peek() {
                Some('}') => {
                    self.bump();
                    break;
                }
                Some(_) => {
                    let start = self.pos;
                    if self.peek() == Some('=') {
                        self.bump();
                        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                            self.bump();
                        }
                    } else {
                        self.ident()?;
                    }
                    let key = self.src.get(start..self.pos).unwrap_or_default().to_owned();
                    if key == "=" {
                        return Err(ParseError::Syntax(start));
                    }
                    self.spaces();
                    self.eat('{')?;
                    let body = self.message(depth.saturating_add(1))?;
                    self.eat('}')?;
                    cases.push((key, body));
                }
                None => return Err(ParseError::Syntax(self.pos)),
            }
        }
        if cases.is_empty() {
            return Err(ParseError::Syntax(self.pos));
        }
        Ok(cases)
    }
}

impl Message {
    /// Parses a message.
    ///
    /// # Errors
    ///
    /// When the message is too long, nested too deeply or malformed.
    pub fn parse(src: &str) -> Result<Self, ParseError> {
        if src.len() > MAX_LEN {
            return Err(ParseError::TooLong);
        }
        let mut p = Parser { src, pos: 0 };
        let m = p.message(0)?;
        if p.pos != src.len() {
            return Err(ParseError::Syntax(p.pos));
        }
        Ok(m)
    }

    /// Every argument name the message uses, at any depth.
    #[must_use]
    pub fn arguments(&self) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        self.collect(&mut out);
        out
    }

    fn collect(&self, out: &mut BTreeSet<String>) {
        for part in &self.0 {
            match part {
                Part::Text(_) => {}
                Part::Arg(a) => {
                    out.insert(a.clone());
                }
                Part::Plural { arg, cases } => {
                    out.insert(arg.clone());
                    for (_, m) in cases {
                        m.collect(out);
                    }
                }
                Part::Select { arg, cases } => {
                    out.insert(arg.clone());
                    for (_, m) in cases {
                        m.collect(out);
                    }
                }
            }
        }
    }

    /// Formats the message. An argument that was not given is left as
    /// `{name}`, so a mistake shows instead of silently disappearing.
    #[must_use]
    pub fn format(&self, args: &[(&str, Arg<'_>)], locale: &dyn Locale) -> String {
        let mut out = String::new();
        self.write(args, locale, &mut out);
        out
    }

    fn write(&self, args: &[(&str, Arg<'_>)], locale: &dyn Locale, out: &mut String) {
        let find = |name: &str| args.iter().find(|(n, _)| *n == name).map(|(_, a)| *a);
        for part in &self.0 {
            match part {
                Part::Text(t) => out.push_str(t),
                Part::Arg(name) => match find(name) {
                    Some(Arg::Text(s)) => locale.text(s, out),
                    Some(Arg::Count(n)) => out.push_str(&locale.number(n)),
                    None => {
                        out.push('{');
                        out.push_str(name);
                        out.push('}');
                    }
                },
                Part::Plural { arg, cases } => {
                    let n = match find(arg) {
                        Some(Arg::Count(n)) => n,
                        Some(Arg::Text(s)) => s.parse().unwrap_or(0),
                        None => 0,
                    };
                    let category = locale.plural(n);
                    let chosen = cases
                        .iter()
                        .find(|(k, _)| *k == Key::Exact(n))
                        .or_else(|| {
                            cases
                                .iter()
                                .find(|(k, _)| matches!(k, Key::Category(c) if c == category))
                        })
                        .or_else(|| {
                            cases
                                .iter()
                                .find(|(k, _)| matches!(k, Key::Category(c) if c == "other"))
                        });
                    if let Some((_, m)) = chosen {
                        m.write(args, locale, out);
                    }
                }
                Part::Select { arg, cases } => {
                    let value = match find(arg) {
                        Some(Arg::Text(s)) => s.to_owned(),
                        Some(Arg::Count(n)) => n.to_string(),
                        None => String::new(),
                    };
                    let chosen = cases
                        .iter()
                        .find(|(k, _)| *k == value)
                        .or_else(|| cases.iter().find(|(k, _)| k == "other"));
                    if let Some((_, m)) = chosen {
                        m.write(args, locale, out);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// English-like rules, ASCII digits, no isolation.
    struct En;
    impl Locale for En {
        fn plural(&self, n: u64) -> &'static str {
            if n == 1 { "one" } else { "other" }
        }
        fn number(&self, n: u64) -> String {
            n.to_string()
        }
        fn text(&self, s: &str, out: &mut String) {
            out.push_str(s);
        }
    }

    fn fmt(src: &str, args: &[(&str, Arg<'_>)]) -> String {
        Message::parse(src)
            .map(|m| m.format(args, &En))
            .unwrap_or_else(|e| format!("ERR {e}"))
    }

    #[test]
    fn plain_text_and_apostrophes_stay() {
        assert_eq!(fmt("L'envoi n'a pas abouti", &[]), "L'envoi n'a pas abouti");
    }

    #[test]
    fn placeholders() {
        assert_eq!(
            fmt("Code: {code}", &[("code", Arg::Text("7-harbor-rocket"))]),
            "Code: 7-harbor-rocket"
        );
        assert_eq!(fmt("Code: {code}", &[]), "Code: {code}");
    }

    #[test]
    fn plurals_exact_then_category_then_other() {
        let m = "{count, plural, =0{none} =1{one file} few{few} other{{count} files}}";
        assert_eq!(fmt(m, &[("count", Arg::Count(0))]), "none");
        assert_eq!(fmt(m, &[("count", Arg::Count(1))]), "one file");
        assert_eq!(fmt(m, &[("count", Arg::Count(7))]), "7 files");
    }

    #[test]
    fn plural_inside_a_sentence_with_another_argument() {
        let m = "{count, plural, =1{1 item} other{{count} items}} · {size}";
        assert_eq!(
            fmt(m, &[("count", Arg::Count(3)), ("size", Arg::Text("2 MB"))]),
            "3 items · 2 MB"
        );
    }

    #[test]
    fn select() {
        let m = "{kind, select, file{a file} folder{a folder} other{something}}";
        assert_eq!(fmt(m, &[("kind", Arg::Text("folder"))]), "a folder");
        assert_eq!(fmt(m, &[("kind", Arg::Text("x"))]), "something");
    }

    #[test]
    fn arguments_at_any_depth() {
        let m = Message::parse("{n, plural, =1{{a}} other{{b} {n}}} {c}").ok();
        let args: Vec<String> = m
            .map(|m| m.arguments().into_iter().collect())
            .unwrap_or_default();
        assert_eq!(args, ["a", "b", "c", "n"]);
    }

    #[test]
    fn rejects_malformed() {
        for bad in [
            "{",
            "}",
            "{}",
            "{1x}",
            "{n, plural, =1{x}}",
            "{n, plural, one{x}",
            "{n, plural, lots{x} other{y}}",
            "{n, plural, ={x} other{y}}",
            "{n, choice, other{y}}",
            "{n, select, a{x}}",
            "{n plural}",
        ] {
            assert!(Message::parse(bad).is_err(), "{bad} should be rejected");
        }
    }

    #[test]
    fn bounds_length_and_depth() {
        assert_eq!(
            Message::parse(&"a".repeat(MAX_LEN + 1)),
            Err(ParseError::TooLong)
        );
        let mut deep = String::from("x");
        for _ in 0..=MAX_DEPTH {
            deep = format!("{{n, select, other{{{deep}}}}}");
        }
        assert_eq!(Message::parse(&deep), Err(ParseError::TooDeep));
    }
}
