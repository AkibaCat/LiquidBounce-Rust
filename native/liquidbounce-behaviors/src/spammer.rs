// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleSpammer and TextFileReaders from pinned LiquidBounce ae0eb396.
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    time::SystemTime,
};

pub const SOURCES: [&str; 2] = ["Setting", "File"];
pub const PATTERNS: [&str; 2] = ["Random", "Linear"];
pub const CONVERTERS: [&str; 4] = ["None", "Leet", "Random Case", "Random Space"];
#[derive(Clone, Debug, PartialEq)]
pub struct Spammer {
    pub delay: [f32; 2],
    pub mps: [i32; 2],
    pub source: u8,
    pub messages: Vec<String>,
    pub file: String,
    pub pattern: u8,
    pub converter: u8,
    pub custom_formatter: bool,
}
impl Default for Spammer {
    fn default() -> Self {
        Self {
            delay: [2.0, 4.0],
            mps: [1, 1],
            source: 0,
            file: String::new(),
            pattern: 0,
            converter: 1,
            custom_formatter: false,
            messages: [
                "LiquidBounce Nextgen | CCBlueX on [youtube] | liquidbounce{.net}",
                "I'm using LiquidBounce Nextgen and you should too!",
                "Check out LiquidBounce Nextgen - the best Minecraft client!",
                "Tired of losing? Try LiquidBounce Nextgen!",
            ]
            .map(str::to_owned)
            .into(),
        }
    }
}
#[derive(Default, Debug)]
pub struct FileIndex {
    path: PathBuf,
    size: u64,
    modified: Option<SystemTime>,
    starts: Vec<u64>,
}
impl FileIndex {
    pub fn next(&mut self, path: &Path, index: usize) -> io::Result<String> {
        let meta = path.metadata()?;
        if self.path != path || self.size != meta.len() || self.modified != meta.modified().ok() {
            let mut reader = BufReader::new(File::open(path)?);
            let mut starts = Vec::new();
            let mut pos = 0u64;
            let mut buf = [0u8; 8192];
            loop {
                let n = reader.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                if pos == 0 {
                    starts.push(0);
                }
                for (i, b) in buf[..n].iter().enumerate() {
                    if *b == b'\n' {
                        starts.push(pos + i as u64 + 1);
                    }
                }
                pos += n as u64;
            }
            if starts.last().is_some_and(|v| *v >= pos) {
                starts.pop();
            }
            self.path = path.to_owned();
            self.size = meta.len();
            self.modified = meta.modified().ok();
            self.starts = starts;
        }
        if self.starts.is_empty() {
            return Err(io::Error::other("empty-source"));
        }
        let mut f = BufReader::new(File::open(path)?);
        f.seek(SeekFrom::Start(self.starts[index % self.starts.len()]))?;
        let mut bytes = Vec::new();
        f.read_until(b'\n', &mut bytes)?;
        if bytes.last() == Some(&b'\n') {
            bytes.pop();
            while bytes.last() == Some(&b'\r') {
                bytes.pop();
            }
        }
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
    pub fn len(&self) -> usize {
        self.starts.len()
    }
}
pub fn convert(text: &str, mode: u8, mut boolean: impl FnMut() -> bool) -> String {
    match mode {
        1 => text
            .chars()
            .map(|c| match c {
                'o' => '0',
                'l' => '1',
                'e' => '3',
                'a' => '4',
                't' => '7',
                's' => 'Z',
                _ => c,
            })
            .collect(),
        2 => text
            .chars()
            .map(|c| {
                if boolean() {
                    c.to_uppercase().next().unwrap_or(c)
                } else {
                    c.to_lowercase().next().unwrap_or(c)
                }
            })
            .collect(),
        3 => {
            let mut s = String::new();
            for c in text.chars() {
                s.push(c);
                if boolean() {
                    s.push(' ');
                }
            }
            s
        }
        _ => text.into(),
    }
}
fn replace_each(text: &str, token: &str, mut value: impl FnMut() -> String) -> String {
    let mut result = String::new();
    let mut rest = text;
    while let Some(i) = rest.find(token) {
        result.push_str(&rest[..i]);
        result.push_str(&value());
        rest = &rest[i + token.len()..];
    }
    result.push_str(rest);
    result
}
fn alpha(random: &mut impl FnMut(i32, i32) -> i32, min: i32, max: i32) -> String {
    let n = random(min, max);
    (0..n)
        .map(|_| {
            let c = random(0, 51) as u8;
            if c < 26 {
                (b'A' + c) as char
            } else {
                (b'a' + c - 26) as char
            }
        })
        .collect()
}
pub fn format_message(
    s: &Spammer,
    text: &str,
    players: &[String],
    mut integer: impl FnMut(i32, i32) -> i32,
    mut unit: impl FnMut() -> f32,
) -> String {
    let body = if s.custom_formatter {
        let text = replace_each(text, "%f", || unit().to_string());
        let text = replace_each(&text, "%i", || integer(0, 9999).to_string());
        let text = replace_each(&text, "%s", || alpha(&mut integer, 4, 6));
        if players.is_empty() {
            text
        } else {
            replace_each(&text, "@a", || {
                players[integer(0, players.len() as i32 - 1) as usize].clone()
            })
        }
    } else {
        let prefix = alpha(&mut integer, 1, 4);
        format!("[{prefix}] {}", convert(text, 2, || integer(0, 1) == 1))
    };
    convert(&body, s.converter, || integer(0, 1) == 1)
}
pub fn utf16_len(text: &str) -> usize {
    text.encode_utf16().count()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn file_reload_format_and_utf16_limit_smoke() {
        let root = std::env::temp_dir().join(format!("lb-spammer-smoke-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("messages.txt");
        std::fs::write(&path, "你好\r\n\nthird\n").unwrap();
        let mut f = FileIndex::default();
        assert_eq!(f.next(&path, 0).unwrap(), "你好");
        assert_eq!(f.len(), 3);
        assert_eq!(f.next(&path, 1).unwrap(), "");
        assert_eq!(f.next(&path, 5).unwrap(), "third");
        std::fs::write(&path, "updated file").unwrap();
        assert_eq!(f.next(&path, 1).unwrap(), "updated file");
        let s = Spammer {
            custom_formatter: true,
            converter: 1,
            ..Default::default()
        };
        assert_eq!(
            format_message(&s, "@a %i %s %f", &["Alice".into()], |a, _| a, || 0.5),
            "A1ic3 0 AAAA 0.5"
        );
        assert_eq!(utf16_len("😀"), 2);
        assert_eq!(convert("toast", 1, || false), "704Z7");
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
}
