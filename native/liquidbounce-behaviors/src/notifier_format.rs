// SPDX-License-Identifier: GPL-3.0-or-later
//! Java Formatter for Notifier String, enum-name and Integer arguments.
use crate::notifier::Arg;
fn number(bytes: &[u8], at: &mut usize) -> Result<Option<usize>, String> {
    let start = *at;
    let mut value = 0usize;
    while let Some(c @ b'0'..=b'9') = bytes.get(*at).copied() {
        value = value
            .checked_mul(10)
            .and_then(|v| v.checked_add(usize::from(c - b'0')))
            .ok_or("format number overflow")?;
        if value > i32::MAX as usize {
            return Err("format number overflow".into());
        }
        *at += 1;
    }
    Ok((*at != start).then_some(value))
}
pub fn format_message(pattern: &str, args: &[Arg]) -> Result<String, String> {
    let bytes = pattern.as_bytes();
    let mut out = String::new();
    let (mut at, mut ordinary, mut previous) = (0, 0usize, None);
    while at < bytes.len() {
        if bytes[at] != b'%' {
            let end = pattern[at..].find('%').map_or(bytes.len(), |n| at + n);
            out.push_str(&pattern[at..end]);
            at = end;
            continue;
        }
        at += 1;
        let saved = at;
        let possible_index = number(bytes, &mut at)?;
        let explicit = if bytes.get(at) == Some(&b'$') {
            at += 1;
            Some(
                possible_index
                    .filter(|i| *i > 0)
                    .ok_or("invalid argument index")?
                    - 1,
            )
        } else {
            at = saved;
            None
        };
        let mut flags = String::new();
        while bytes.get(at).is_some_and(|c| b"-#+ 0,(<".contains(c)) {
            let c = bytes[at] as char;
            if flags.contains(c) {
                return Err("duplicate format flag".into());
            }
            flags.push(c);
            at += 1;
        }
        let width = number(bytes, &mut at)?;
        let precision = if bytes.get(at) == Some(&b'.') {
            at += 1;
            Some(number(bytes, &mut at)?.ok_or("missing format precision")?)
        } else {
            None
        };
        let conversion = bytes.get(at).copied().ok_or("missing format conversion")? as char;
        at += 1;
        if conversion == '%' || conversion == 'n' {
            if precision.is_some() || flags.chars().any(|c| c != '-') {
                return Err("invalid text conversion flags".into());
            }
            if conversion == 'n' {
                if width.is_some() || !flags.is_empty() {
                    return Err("newline does not accept flags or width".into());
                }
                out.push_str(if cfg!(windows) { "\r\n" } else { "\n" });
                continue;
            }
            if flags.contains('-') && width.is_none() {
                return Err("left alignment needs a width".into());
            }
            let padding = width.unwrap_or(1).saturating_sub(1);
            if !flags.contains('-') {
                out.extend(std::iter::repeat_n(' ', padding));
            }
            out.push('%');
            if flags.contains('-') {
                out.extend(std::iter::repeat_n(' ', padding));
            }
            continue;
        }
        if !matches!(
            conversion,
            's' | 'S' | 'b' | 'B' | 'h' | 'H' | 'd' | 'o' | 'x' | 'X' | 'c' | 'C'
        ) {
            return Err(format!("conversion {conversion} does not accept a String"));
        }
        let numeric = matches!(conversion, 'd' | 'o' | 'x' | 'X');
        if !numeric
            && (flags.chars().any(|c| c != '-' && c != '<')
                || flags.contains('-') && width.is_none())
        {
            return Err("invalid String conversion flags".into());
        }
        let index = if flags.contains('<') {
            previous.ok_or("no previous format argument")?
        } else if let Some(index) = explicit {
            index
        } else {
            let index = ordinary;
            ordinary += 1;
            index
        };
        let argument = args.get(index).ok_or("missing format argument")?;
        let name = match argument {
            Arg::Text(s) => s.clone(),
            Arg::Int(n) => n.to_string(),
        };
        if numeric {
            out.push_str(&integer(argument, conversion, &flags, width, precision)?);
            previous = Some(index);
            continue;
        }
        if matches!(conversion, 'c' | 'C') {
            let Arg::Int(n) = argument else {
                return Err("character conversion requires an Integer".into());
            };
            let ch = char::from_u32(*n as u32).ok_or("invalid Unicode code point")?;
            let value = if conversion == 'C' {
                ch.to_uppercase().collect::<String>()
            } else {
                ch.to_string()
            };
            if precision.is_some() {
                return Err("character precision is not supported".into());
            }
            let pad = width
                .unwrap_or(0)
                .saturating_sub(value.encode_utf16().count());
            if !flags.contains('-') {
                out.extend(std::iter::repeat_n(' ', pad));
            }
            out.push_str(&value);
            if flags.contains('-') {
                out.extend(std::iter::repeat_n(' ', pad));
            }
            previous = Some(index);
            continue;
        }
        previous = Some(index);
        let value = match conversion {
            's' | 'S' => name.to_owned(),
            'b' | 'B' => "true".into(),
            _ => format!(
                "{:x}",
                match argument {
                    Arg::Int(n) => *n as u32,
                    Arg::Text(s) => s
                        .encode_utf16()
                        .fold(0u32, |h, c| h.wrapping_mul(31).wrapping_add(u32::from(c))),
                }
            ),
        };
        // Formatter precision and width count UTF-16 code units, before uppercasing.
        let mut value: Vec<u16> = value.encode_utf16().collect();
        if let Some(precision) = precision {
            value.truncate(precision);
        }
        let value: String = char::decode_utf16(value)
            .map(|c| c.unwrap_or('?'))
            .collect();
        let value = if conversion.is_ascii_uppercase() {
            value.to_uppercase()
        } else {
            value
        };
        let padding = width
            .unwrap_or(0)
            .saturating_sub(value.encode_utf16().count());
        if !flags.contains('-') {
            out.extend(std::iter::repeat_n(' ', padding));
        }
        out.push_str(&value);
        if flags.contains('-') {
            out.extend(std::iter::repeat_n(' ', padding));
        }
    }
    Ok(out)
}

fn integer(
    argument: &Arg,
    conversion: char,
    flags: &str,
    width: Option<usize>,
    precision: Option<usize>,
) -> Result<String, String> {
    let Arg::Int(n) = argument else {
        return Err("integer conversion requires an Integer".into());
    };
    if precision.is_some() {
        return Err("integer precision is not supported".into());
    }
    if flags.contains('-') && flags.contains('0') || flags.contains('+') && flags.contains(' ') {
        return Err("conflicting integer flags".into());
    }
    if (flags.contains('-') || flags.contains('0')) && width.is_none() {
        return Err("integer alignment needs width".into());
    }
    let valid = if conversion == 'd' { "-+ 0,(<" } else { "-#0<" };
    if flags.chars().any(|c| !valid.contains(c)) {
        return Err("invalid integer flags".into());
    }
    let mut value = match conversion {
        'd' => i64::from(*n).abs().to_string(),
        'o' => format!("{:o}", *n as u32),
        _ => format!("{:x}", *n as u32),
    };
    if flags.contains(',') {
        let mut grouped = String::new();
        for (i, c) in value.chars().enumerate() {
            if i > 0 && (value.len() - i) % 3 == 0 {
                grouped.push(',');
            }
            grouped.push(c);
        }
        value = grouped;
    }
    let negative = conversion == 'd' && *n < 0;
    let parentheses = negative && flags.contains('(');
    let prefix = if parentheses {
        "("
    } else if negative {
        "-"
    } else if conversion == 'd' && flags.contains('+') {
        "+"
    } else if conversion == 'd' && flags.contains(' ') {
        " "
    } else if conversion == 'o' && flags.contains('#') {
        "0"
    } else if conversion != 'd' && flags.contains('#') {
        "0x"
    } else {
        ""
    };
    let suffix = if parentheses { ")" } else { "" };
    let pad = width
        .unwrap_or(0)
        .saturating_sub(prefix.len() + value.len() + suffix.len());
    let mut out = String::new();
    if !flags.contains('-') && !flags.contains('0') {
        out.extend(std::iter::repeat_n(' ', pad));
    }
    out.push_str(prefix);
    if flags.contains('0') {
        out.extend(std::iter::repeat_n('0', pad));
    }
    out.push_str(&value);
    out.push_str(suffix);
    if flags.contains('-') {
        out.extend(std::iter::repeat_n(' ', pad));
    }
    if conversion == 'X' {
        out = out.to_uppercase();
    }
    Ok(out)
}
