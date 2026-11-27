// SPDX-License-Identifier: GPL-3.0-or-later
//! Query only each landing box, highest first; intermediate blocks are skipped.
pub fn highest_clear<E>(
    maximum: i32,
    mut is_clear: impl FnMut(i32) -> Result<bool, E>,
) -> Result<i32, E> {
    for height in (1..=maximum).rev() {
        if is_clear(height)? {
            return Ok(height);
        }
    }
    Ok(0)
}
