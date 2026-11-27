// SPDX-License-Identifier: GPL-3.0-or-later
//! The source BookBuilder's 14-line/114-pixel algorithm, including overflow rules.
pub fn build(
    mut chars: impl Iterator<Item = u32>,
    page_amount: i32,
    mut width: impl FnMut(u16) -> f32,
) -> Vec<Vec<u16>> {
    let mut pages = Vec::new();
    let mut page = Vec::new();
    let mut line = 0;
    let mut line_width = 0f32;
    while let Some(code) = chars.next() {
        let c = code as u16;
        // Source filters CR after narrowing to Char, before the builder.
        if c == 13 {
            continue;
        }
        if line_width == 0. && c == 32 {
            continue;
        }
        if c == 10 {
            page.push(10);
            line_width = 0.;
            line += 1;
        } else {
            let w = width(c);
            if line_width + w > 114. {
                line += 1;
                line_width = w;
                page.push(10);
                if line == 14 {
                    page.push(32);
                }
            } else {
                line_width += w;
                page.push(c);
            }
        }
        if line == 14 {
            pages.push(std::mem::take(&mut page));
            line = 0;
            if pages.len() as i32 >= page_amount {
                break;
            }
            if c != 10 && c != 13 {
                page.push(c);
            }
        }
    }
    if !page.is_empty() && (pages.len() as i32) < page_amount {
        pages.push(page);
    }
    pages
}
/// Netty UTF-8 writing uses '?' for an unpaired UTF-16 surrogate.
pub fn wire_text(units: &[u16]) -> String {
    char::decode_utf16(units.iter().copied())
        .map(|c| c.unwrap_or('?'))
        .collect()
}
pub fn java_whitespace(c: u32) -> bool {
    matches!(c,9..=13|28..=32|0x1680|0x2000..=0x2006|0x2008..=0x200a|0x2028|0x2029|0x205f|0x3000)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pagination_and_utf16_smoke() {
        let pages = build("  ab\r\ncd".chars().map(|c| c as u32), 2, |_| 6.);
        assert_eq!(pages, vec!["ab\ncd".encode_utf16().collect::<Vec<_>>()]);
        let pages = build(std::iter::repeat(b'x' as u32), 1, |_| 114.);
        assert_eq!(pages.len(), 1);
        assert_eq!(
            pages[0],
            "x\n\n\n\n\n\n\n\n\n\n\n\n\n\n "
                .encode_utf16()
                .collect::<Vec<_>>()
        );
        assert_eq!(wire_text(&[0xd800, 65, 0xd83d, 0xde00]), "?A😀");
        assert!(!java_whitespace(0xa0));
        assert!(java_whitespace(0x2000));
    }
}
