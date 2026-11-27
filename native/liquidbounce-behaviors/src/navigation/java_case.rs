// SPDX-License-Identifier: GPL-3.0-or-later
//! Java 25 String.equalsIgnoreCase for valid Unicode scalar strings. Minecraft
//! profile names and Rust strings cannot contain isolated UTF-16 surrogates.
#[path = "java_case_data.rs"]
mod data;
fn fold(value: char) -> u32 {
    let point = u32::from(value);
    data::SIMPLE_FOLD
        .binary_search_by_key(&point, |&(source, _)| source)
        .map_or(point, |index| data::SIMPLE_FOLD[index].1)
}
pub fn equals_ignore_case(left: &str, right: &str) -> bool {
    left.encode_utf16().count() == right.encode_utf16().count()
        && left.chars().map(fold).eq(right.chars().map(fold))
}
