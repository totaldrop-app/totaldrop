//! Translations come from outside contributors: the message parser must
//! never panic, and formatting what it accepts must not either.
#![no_main]

use libfuzzer_sys::fuzz_target;
use totaldrop_l10n::message::{Arg, Locale, Message};

struct Plain;
impl Locale for Plain {
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

fuzz_target!(|data: &[u8]| {
    if let Ok(src) = std::str::from_utf8(data)
        && let Ok(m) = Message::parse(src)
    {
        let _ = m.arguments();
        let _ = m.format(&[("count", Arg::Count(3)), ("name", Arg::Text("x"))], &Plain);
    }
});
