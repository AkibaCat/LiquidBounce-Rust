// SPDX-License-Identifier: GPL-3.0-or-later
//! BetterChat's foreground cursor hit test and clipped wrapped-message rectangle.
pub fn bounds<T: PartialEq>(keys: &[T], index: usize) -> Option<(usize, usize)> {
    let key = keys.get(index)?;
    let mut start = index;
    let mut end = index;
    while start > 0 && &keys[start - 1] == key {
        start -= 1;
    }
    while end + 1 < keys.len() && &keys[end + 1] == key {
        end += 1;
    }
    Some((start, end))
}
pub fn hit(
    total: usize,
    scroll: usize,
    page: usize,
    width: i32,
    height: i32,
    scale: f64,
    line_height: i32,
    x: f64,
    y: f64,
) -> Option<usize> {
    if scale <= 0. || line_height <= 0 {
        return None;
    }
    let w = (f64::from(width) / scale).ceil();
    let x = x / scale - 4.;
    if x < 0. || x > w {
        return None;
    }
    let bottom = (f64::from(height - 40) / scale).floor();
    let local = bottom - y / scale;
    if local < 0. {
        return None;
    }
    let row = (local / f64::from(line_height)).floor() as usize;
    let visible = total.saturating_sub(scroll).min(page);
    (row < visible).then_some(scroll + row)
}
pub fn rect(
    bounds: (usize, usize),
    scroll: usize,
    total: usize,
    page: usize,
    width: i32,
    height: i32,
    scale: f64,
    line_height: i32,
) -> Option<[i32; 4]> {
    if scale <= 0. || line_height <= 0 {
        return None;
    }
    let visible = total.saturating_sub(scroll).min(page);
    if visible == 0 {
        return None;
    }
    let start = bounds.0.max(scroll);
    let end = bounds.1.min(scroll + visible - 1);
    if start > end {
        return None;
    }
    let w = (f64::from(width) / scale).ceil();
    let bottom = (f64::from(height - 40) / scale).floor();
    Some([
        (4. * scale).floor() as i32,
        ((bottom - (end - scroll + 1) as f64 * f64::from(line_height)) * scale).floor() as i32,
        ((w + 4.) * scale).ceil() as i32,
        ((bottom - (start - scroll) as f64 * f64::from(line_height)) * scale).ceil() as i32,
    ])
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wrapped_message_bounds_and_scaled_clipping() {
        assert_eq!(bounds(&[9, 9, 8, 8, 8, 7], 3), Some((2, 4)));
        assert_eq!(hit(20, 3, 4, 320, 240, 0.5, 9, 6., 195.), Some(4));
        assert_eq!(
            rect((2, 7), 3, 20, 4, 320, 240, 0.5, 9),
            Some([2, 182, 322, 200])
        );
        assert_eq!(hit(20, 0, 4, 320, 240, 1., 9, 3., 195.), None);
        assert_eq!(hit(20, 0, 4, 320, 240, 0., 9, 10., 195.), None);
    }
}
