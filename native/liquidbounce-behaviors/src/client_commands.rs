// SPDX-License-Identifier: GPL-3.0-or-later
//! CommandManager's normalization and Brigadier ClientStringArgumentType rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub prefix: String,
    pub hint_count: i32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            prefix: ".".into(),
            hint_count: 5,
        }
    }
}
pub fn normalize(input: &str) -> String {
    let mut out = String::new();
    let (mut quote, mut escaped, mut space) = (false, false, false);
    for c in input.trim().chars() {
        if escaped {
            out.push(c);
            escaped = false;
            space = false;
            continue;
        }
        match c {
            '\\' => {
                out.push(c);
                escaped = true;
                space = false;
            }
            '"' => {
                quote = !quote;
                out.push(c);
                space = false;
            }
            ' ' if !quote => {
                if !space {
                    out.push(c);
                }
                space = true;
            }
            _ => {
                out.push(c);
                space = false;
            }
        }
    }
    out
}
pub struct Reader {
    input: String,
    pos: usize,
}
impl Reader {
    pub fn new(input: &str) -> Self {
        Self {
            input: normalize(input),
            pos: 0,
        }
    }
    fn skip(&mut self) {
        while self.input.as_bytes().get(self.pos) == Some(&b' ') {
            self.pos += 1;
        }
    }
    pub fn word(&mut self) -> Option<String> {
        self.skip();
        let start = self.pos;
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos] != b' ' {
            self.pos += 1;
        }
        (self.pos > start).then(|| self.input[start..self.pos].to_owned())
    }
    pub fn string(&mut self) -> Result<Option<String>, String> {
        self.skip();
        let Some(quote) = self.input[self.pos..].chars().next() else {
            return Ok(None);
        };
        if !matches!(quote, '"' | '\'') {
            return Ok(self.word());
        }
        self.pos += 1;
        let (mut out, mut escaped) = (String::new(), false);
        while let Some(c) = self.input[self.pos..].chars().next() {
            self.pos += c.len_utf8();
            if escaped {
                if c != quote && c != '\\' {
                    return Err("Invalid escape in quoted command argument".into());
                }
                out.push(c);
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == quote {
                if self.pos < self.input.len() && self.input.as_bytes()[self.pos] != b' ' {
                    return Err("Expected whitespace after quoted argument".into());
                }
                return Ok(Some(out));
            } else {
                out.push(c);
            }
        }
        Err("Unclosed quoted command argument".into())
    }
    pub fn greedy(&mut self) -> String {
        self.skip();
        let out = self.input[self.pos..].to_owned();
        self.pos = self.input.len();
        out
    }
    pub fn finish(&mut self) -> Result<(), String> {
        self.skip();
        if self.pos == self.input.len() {
            Ok(())
        } else {
            Err("Unexpected trailing command input".into())
        }
    }
    pub fn required(&mut self) -> Result<String, String> {
        self.word().ok_or_else(|| "Missing command argument".into())
    }
}
