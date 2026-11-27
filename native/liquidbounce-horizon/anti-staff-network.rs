// SPDX-License-Identifier: GPL-3.0-or-later
//! Actual pinned ClientCdn endpoint; called exclusively from the actor's worker.
use std::io::Read;
#[derive(Debug)]
pub enum FetchError {
    Http(u16),
    Transport(String),
    Body(String),
}
impl FetchError {
    pub fn code(&self) -> String {
        match self {
            Self::Http(code) => code.to_string(),
            Self::Transport(e) => format!("TransportError: {e}"),
            Self::Body(e) => format!("IoError: {e}"),
        }
    }
}
pub fn load(domain: &str) -> Result<String, FetchError> {
    // URL path segment escaping, including bracketless IPv6 and percent zones.
    let mut escaped = String::new();
    for b in domain.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            escaped.push(b as char);
        } else {
            escaped.push_str(&format!("%{b:02X}"));
        }
    }
    let response = ureq::get(&format!(
        "https://cloud.liquidbounce.net/LiquidBounce/staffs/{escaped}"
    ))
    .set("User-Agent", "LiquidBounce/Rust-26.3")
    .call()
    .map_err(|e| match e {
        ureq::Error::Status(code, _) => FetchError::Http(code),
        ureq::Error::Transport(e) => FetchError::Transport(e.to_string()),
    })?;
    let mut text = String::new();
    response
        .into_reader()
        .read_to_string(&mut text)
        .map_err(|e| FetchError::Body(e.to_string()))?;
    Ok(text)
}
