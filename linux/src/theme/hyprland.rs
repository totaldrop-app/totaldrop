//! Hyprland.
//!
//! The compositor draws the window frame, so the app has no decoration of
//! its own. What the app does take from Hyprland is its corner rounding,
//! so cards inside the window have the same corners as the window itself.
//! The value is read again whenever Hyprland reloads its configuration.

use std::path::PathBuf;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

fn socket_dir() -> Option<PathBuf> {
    let sig = std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE")?;
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")?;
    Some(PathBuf::from(runtime).join("hypr").join(sig))
}

/// Whether this session runs under Hyprland.
pub fn running() -> bool {
    socket_dir().is_some_and(|d| d.join(".socket.sock").exists())
}

/// `decoration:rounding`, in pixels.
pub async fn rounding() -> Option<i32> {
    let mut s = UnixStream::connect(socket_dir()?.join(".socket.sock"))
        .await
        .ok()?;
    s.write_all(b"j/getoption decoration:rounding").await.ok()?;
    let mut reply = String::new();
    s.read_to_string(&mut reply).await.ok()?;
    parse_rounding(&reply)
}

fn parse_rounding(reply: &str) -> Option<i32> {
    let v: serde_json::Value = serde_json::from_str(reply).ok()?;
    let n = v.get("int")?.as_i64()?;
    i32::try_from(n.clamp(0, 64)).ok()
}

/// Call `on_reload` each time Hyprland reloads its configuration. Returns
/// when the event socket closes.
pub async fn on_config_reload(mut on_reload: impl FnMut()) {
    let Some(dir) = socket_dir() else { return };
    let Ok(s) = UnixStream::connect(dir.join(".socket2.sock")).await else {
        return;
    };
    let mut lines = BufReader::new(s).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if line.starts_with("configreloaded>>") {
            on_reload();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_getoption() {
        assert_eq!(
            parse_rounding(r#"{"option": "decoration:rounding", "int": 0, "set": true }"#),
            Some(0)
        );
        assert_eq!(parse_rounding(r#"{"int": 10}"#), Some(10));
        assert_eq!(parse_rounding("no such option"), None);
    }
}
