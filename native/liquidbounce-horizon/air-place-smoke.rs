// SPDX-License-Identifier: GPL-3.0-or-later
#[path = "../liquidbounce-behaviors/src/easing.rs"]
mod easing;
#[path = "../liquidbounce-behaviors/src/numeric.rs"]
mod numeric;
pub use easing::Easing;
#[path = "../liquidbounce-behaviors/src/camera_clip.rs"]
mod camera_clip;
#[path = "../liquidbounce-behaviors/src/air_place.rs"]
mod model;
#[test]
fn air_place_range_and_scroll() {
    use model::*;
    assert_eq!(target([0., 1., 0.], [0., 1., 9.], 3.), [0., 1., 3.]);
    assert_eq!(target([2., 3., 4.], [2., 3., 4.], 4.5), [2., 3., 4.]);
    let diagonal = target([0.; 3], [3., 0., 4.], 2.5);
    assert_eq!(diagonal, [1.5, 0., 2.]);
    let mut s = Settings::default();
    assert!(!s.can_scroll(true, true));
    s.custom_range = true;
    s.scroll(true, false, 100.);
    assert_eq!(s.range, 3.);
    s.scroll(true, true, 1.);
    assert_eq!(s.range, 3.5);
    s.scroll(true, true, 100.);
    assert_eq!(s.range, 4.5);
    s.scroll(true, true, -100.);
    assert_eq!(s.range, 1.);
    s.modifier = camera_clip::ModifierKey::from_name("key.keyboard.unknown").unwrap();
    assert!(s.can_scroll(true, false));
    s.scroll_adjust = false;
    assert!(!s.can_scroll(true, true));
}
