// GUICloser corrections: observable facts generated from the pinned JDK25.
// The generation program and its evidence are part of the parent project.
use crate::types::PredefinedClass;
use alloc::format;
use alloc::string::String;
include!("../jdk25/manifest.rs");
static PROPERTY_DATA: &[u8] = include_bytes!("../jdk25/properties.bin");
static CHARACTER_DATA: &[u8] = include_bytes!("../jdk25/characters.bin");
static CASE_DATA: &[u8] = include_bytes!("../jdk25/case.bin");
static NAME_DATA: &[u8] = include_bytes!("../jdk25/names.bin");
static NAME_INDEX: &[u8] = include_bytes!("../jdk25/names-index.bin");
fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_be_bytes(bytes[offset..offset + 8].try_into().unwrap())
}
fn facts(c: u32) -> u64 {
    let cp = c;
    let count = CHARACTER_DATA.len() / 12;
    let mut low = 0;
    let mut high = count;
    while low < high {
        let mid = (low + high) / 2;
        if u32_at(CHARACTER_DATA, mid * 12) <= cp {
            low = mid + 1;
        } else {
            high = mid;
        }
    }
    u64_at(CHARACTER_DATA, (low - 1) * 12 + 4)
}
fn mapping(c: u32, offset: usize) -> u32 {
    let cp = c;
    let mut low = 0;
    let mut high = CASE_DATA.len() / 12;
    while low < high {
        let mid = (low + high) / 2;
        match u32_at(CASE_DATA, mid * 12).cmp(&cp) {
            core::cmp::Ordering::Equal => return u32_at(CASE_DATA, mid * 12 + offset),
            core::cmp::Ordering::Less => low = mid + 1,
            core::cmp::Ordering::Greater => high = mid,
        }
    }
    c
}
pub fn simple_upper(c: u32) -> u32 {
    mapping(c, 4)
}
pub fn simple_lower(c: u32) -> u32 {
    mapping(c, 8)
}
pub fn chars_eq_ci(a: u32, b: u32, unicode: bool) -> bool {
    if !unicode {
        return ascii_lower(a) == ascii_lower(b);
    }
    let a = simple_upper(a);
    let b = simple_upper(b);
    a == b || simple_lower(a) == simple_lower(b)
}
pub fn is_line_terminator(c: u32) -> bool {
    matches!(c, 10 | 13 | 133 | 8232 | 8233)
}
pub fn is_linebreak(c: u32) -> bool {
    matches!(c, 10 | 11 | 12 | 13 | 133 | 8232 | 8233)
}
fn property(name: &str) -> Option<(usize, usize)> {
    PROPERTIES
        .binary_search_by_key(&name, |entry| entry.0)
        .ok()
        .map(|i| (PROPERTIES[i].1, PROPERTIES[i].2))
}
fn property_has(location: (usize, usize), c: u32) -> bool {
    let (offset, count) = location;
    let cp = c;
    let mut low = 0;
    let mut high = count;
    while low < high {
        let mid = (low + high) / 2;
        if u32_at(PROPERTY_DATA, offset + mid * 8 + 4) <= cp {
            low = mid + 1;
        } else {
            high = mid;
        }
    }
    low < count && u32_at(PROPERTY_DATA, offset + low * 8) <= cp
}
enum Property {
    Ranges(usize, usize, bool),
    Script(u64),
    Block(u64),
}
fn named_id(entries: &[(&str, u64)], name: &str) -> Option<u64> {
    let name = name.to_uppercase();
    entries
        .binary_search_by_key(&name.as_str(), |entry| entry.0)
        .ok()
        .map(|i| entries[i].1)
}
fn resolve(name: &str, unicode: bool, ci: bool) -> Option<Property> {
    let suffix = if ci { 1 } else { 0 };
    let binary = |key: String| {
        property(&key).map(|(offset, count)| {
            Property::Ranges(
                offset,
                count,
                BMP_PROPERTIES.binary_search(&key.as_str()).is_ok(),
            )
        })
    };
    if let Some((kind, value)) = name.split_once('=') {
        return match kind.to_lowercase().as_str() {
            "sc" | "script" => named_id(SCRIPTS, value).map(Property::Script),
            "blk" | "block" => named_id(BLOCKS, value).map(Property::Block),
            "gc" | "general_category" => binary(format!("D:{value}:{suffix}")),
            _ => None,
        };
    }
    if let Some(value) = name.strip_prefix("In") {
        return named_id(BLOCKS, value).map(Property::Block);
    }
    if let Some(value) = name.strip_prefix("Is") {
        return binary(format!("U:{}:{suffix}", value.to_uppercase()))
            .or_else(|| named_id(SCRIPTS, value).map(Property::Script));
    }
    if unicode {
        if let Some(value) = binary(format!("P:{}:{suffix}", name.to_uppercase())) {
            return Some(value);
        }
    }
    binary(format!("D:{name}:{suffix}"))
}
pub fn is_valid_unicode_property(name: &str, unicode: bool) -> bool {
    resolve(name, unicode, false).is_some()
}
pub fn match_unicode_property_flags(name: &str, c: u32, unicode: bool, ci: bool) -> bool {
    match resolve(name, unicode, ci) {
        Some(Property::Ranges(offset, count, _)) => property_has((offset, count), c),
        Some(Property::Script(id)) => (facts(c) >> 14) & 0x1ff == id,
        Some(Property::Block(id)) => (facts(c) >> 23) & 0x3ff == id,
        None => false,
    }
}
pub fn property_supplementary(name: &str, unicode: bool, ci: bool) -> bool {
    !matches!(
        resolve(name, unicode, ci),
        Some(Property::Ranges(_, _, true))
    )
}
pub fn is_word_char(c: u32, unicode: bool) -> bool {
    if unicode {
        property_has(property("U:WORD:0").unwrap(), c)
    } else {
        (65..=90).contains(&c) || (97..=122).contains(&c) || (48..=57).contains(&c) || c == 95
    }
}
pub fn match_predefined_class(pc: PredefinedClass, c: u32, unicode: bool) -> bool {
    let digit = || {
        if unicode {
            property_has(property("U:DIGIT:0").unwrap(), c)
        } else {
            (48..=57).contains(&c)
        }
    };
    let space = || {
        if unicode {
            property_has(property("U:WHITE_SPACE:0").unwrap(), c)
        } else {
            matches!(c, 32 | 9 | 10 | 13 | 11 | 12)
        }
    };
    let horizontal = || {
        matches!(
            c,
            9 | 32 | 160 | 5760 | 6158 | 8192..=8202 | 8239 | 8287 | 12288
        )
    };
    match pc {
        PredefinedClass::Digit => digit(),
        PredefinedClass::NonDigit => !digit(),
        PredefinedClass::Word => is_word_char(c, unicode),
        PredefinedClass::NonWord => !is_word_char(c, unicode),
        PredefinedClass::Whitespace => space(),
        PredefinedClass::NonWhitespace => !space(),
        PredefinedClass::HorizWhitespace => horizontal(),
        PredefinedClass::NonHorizWhitespace => !horizontal(),
        PredefinedClass::VertWhitespace => is_linebreak(c),
        PredefinedClass::NonVertWhitespace => !is_linebreak(c),
    }
}
pub fn named_character(name: &str) -> Option<u32> {
    let name = name.trim_matches(|c: char| c as u32 <= 0x20).to_uppercase();
    let wanted = name.as_bytes();
    let mut low = 0;
    let mut high = NAME_INDEX.len() / 8;
    while low < high {
        let mid = (low + high) / 2;
        let start = u32_at(NAME_INDEX, mid * 8) as usize;
        let length = NAME_DATA[start..].iter().position(|&b| b == 0).unwrap();
        match NAME_DATA[start..start + length].cmp(wanted) {
            core::cmp::Ordering::Equal => return Some(u32_at(NAME_INDEX, mid * 8 + 4)),
            core::cmp::Ordering::Less => low = mid + 1,
            core::cmp::Ordering::Greater => high = mid,
        }
    }
    None
}
pub fn ascii_lower(c: u32) -> u32 {
    if (65..=90).contains(&c) {
        c + 32
    } else {
        c
    }
}
pub fn ascii_upper(c: u32) -> u32 {
    if (97..=122).contains(&c) {
        c - 32
    } else {
        c
    }
}
pub fn literal_eq_ci(input: u32, pattern: u32, unicode: bool) -> bool {
    if !unicode {
        return ascii_lower(input) == ascii_lower(pattern);
    }
    let upper = simple_upper(pattern);
    let lower = simple_lower(upper);
    if upper == lower {
        return input == pattern;
    }
    input == lower || simple_lower(simple_upper(input)) == lower
}
pub fn is_nonspacing_mark(c: u32) -> bool {
    facts(c) & (1 << 7) != 0
}
pub fn codepoint_at(input: &[u32], pos: usize, end: usize) -> (u32, usize) {
    let high = input[pos];
    if (0xd800..=0xdbff).contains(&high) && pos + 1 < end {
        let low = input[pos + 1];
        if (0xdc00..=0xdfff).contains(&low) {
            return (0x10000 + ((high - 0xd800) << 10) + (low - 0xdc00), 2);
        }
    }
    (high, 1)
}
pub fn codepoint_before(input: &[u32], pos: usize, start: usize) -> (u32, usize) {
    let low = input[pos - 1];
    if (0xdc00..=0xdfff).contains(&low) && pos >= start + 2 {
        let high = input[pos - 2];
        if (0xd800..=0xdbff).contains(&high) {
            return (0x10000 + ((high - 0xd800) << 10) + (low - 0xdc00), 2);
        }
    }
    (low, 1)
}
pub fn next_grapheme(input: &[u32], start: usize, end: usize) -> usize {
    if start >= end {
        return end;
    }
    let (first, width) = codepoint_at(input, start, end);
    let mut previous = facts(first);
    let mut ri = usize::from(previous & 15 == 6);
    let mut pictographic = previous & 15 == 14;
    let mut zwj_pictographic = false;
    let mut consonant = previous & (1 << 6) != 0;
    let mut linker = false;
    let mut pos = start + width;
    while pos < end {
        let (cp, width) = codepoint_at(input, pos, end);
        let current = facts(cp);
        let left = (previous & 15) as usize;
        let right = (current & 15) as usize;
        let conjunct = consonant && linker && current & (1 << 6) != 0;
        let joined = left == 5 && right == 14 && zwj_pictographic;
        let regional = left == 6 && right == 6 && ri % 2 == 1;
        if GRAPHEME_RULES[left][right] && !conjunct && !joined && !regional {
            return pos;
        }
        ri = if right == 6 { ri + 1 } else { 0 };
        zwj_pictographic = right == 5 && pictographic;
        if right != 4 {
            pictographic = right == 14;
        }
        if current & (1 << 6) != 0 {
            consonant = true;
            linker = false;
        } else if current & ((1 << 4) | (1 << 5)) != 0 {
            linker |= consonant && current & (1 << 4) != 0;
        } else {
            consonant = false;
            linker = false;
        }
        previous = current;
        pos += width;
    }
    end
}
pub fn grapheme_boundary(input: &[u32], pos: usize, start: usize, end: usize) -> bool {
    if pos == start || pos == end {
        return true;
    }
    let mut cursor = start;
    while cursor < pos {
        cursor = next_grapheme(input, cursor, end);
    }
    cursor == pos
}
