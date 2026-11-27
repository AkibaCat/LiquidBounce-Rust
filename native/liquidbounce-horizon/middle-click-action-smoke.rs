#[path = "../../native/liquidbounce-behaviors/src/middle_click_action.rs"]
mod middle_click_action;
use middle_click_action::*;
#[test]
fn pearl_release_and_friend_press_edges() {
    let mut m = MiddleClickAction::default();
    let mut c = Context {
        screen_open: false,
        pitch: 0.,
        pickup: true,
        pearl: Some(Slot::Hotbar(2)),
        candidate: true,
        visible: true,
    };
    assert_eq!(m.tick(true, c), Some(Action::ToggleFriend));
    assert_eq!(m.tick(true, c), None);
    c.pickup = false;
    c.candidate = false;
    assert_eq!(m.tick(true, c), None);
    c.pickup = true;
    c.candidate = true;
    assert_eq!(m.tick(true, c), None);
    c.pickup = false;
    assert_eq!(m.tick(true, c), None);
    c.pickup = true;
    c.visible = false;
    assert_eq!(m.tick(true, c), None);
    c.visible = true;
    assert_eq!(m.tick(true, c), None);
    m.set_mode(Mode::Pearl);
    assert_eq!(m.tick(true, c), Some(Action::Select(Slot::Hotbar(2), 1)));
    assert!(m.cancel_pick(true, c.pearl));
    c.pearl = None;
    assert_eq!(m.tick(true, c), None);
    c.pickup = false;
    c.pearl = Some(Slot::Offhand);
    assert_eq!(m.tick(true, c), Some(Action::Use(Slot::Offhand, 1)));
    assert_eq!(Slot::Offhand.hand(), 1);
    c.pickup = true;
    assert_eq!(m.tick(true, c), Some(Action::Select(Slot::Offhand, 1)));
    c.pitch = 85.;
    assert_eq!(m.tick(true, c), None);
    c.pickup = false;
    c.pitch = 0.;
    assert_eq!(m.tick(true, c), None);
    c.pickup = true;
    m.tick(true, c);
    m.world_change();
    c.pickup = false;
    assert_eq!(m.tick(true, c), None);
    c.pickup = true;
    m.tick(true, c);
    m.pearl_disable();
    c.pickup = false;
    assert_eq!(m.tick(true, c), None);
    assert_eq!(Mode::parse("FriendClicker"), Some(Mode::FriendClicker));
    assert_eq!(Mode::Pearl.tag(), "Pearl");
}
