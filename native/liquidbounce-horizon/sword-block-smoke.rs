// SPDX-License-Identifier: GPL-3.0-or-later
#[test]
fn local_sword_animation_uses_actual_offhand_and_physical_key() {
    let mut g = game();
    assert!(!g.client_sword_block_pose());
    g.using = Some((40, 0));
    assert!(g.client_sword_block_pose());
    g.using = Some((0, 0));
    assert!(!g.client_sword_block_pose());
    g.using = None;
    g.right = true;
    g.flags |= 2;
    assert!(g.client_sword_block_pose());
    g.flags |= 32;
    assert!(g.client_sword_block_pose(), "OnlyVisual does not disable visuals");
    g.flags &= !1;
    assert!(!g.client_sword_block_pose());
    g.aura = true;
    assert!(g.client_sword_block_pose());
    g.inventory[0] = Some(item("minecraft:stick"));
    assert!(!g.client_sword_block_pose());
    g.flags |= 1;
    assert!(!g.client_sword_block_pose());
}
#[test]
fn shield_hiding_uses_the_rendered_offhand_stack_and_live_main_hand() {
    let mut g = game();
    assert!(g.client_sword_block_hide_offhand());
    assert!(!g.client_sword_block_hide_stack(Some(&item("minecraft:torch"))));
    g.inventory[0] = Some(item("minecraft:stick"));
    assert!(!g.client_sword_block_hide_offhand());
    g.flags |= 16;
    assert!(g.client_sword_block_hide_offhand());
    g.flags &= !1;
    assert!(!g.client_sword_block_hide_offhand());
    g.owner = false;
    assert_eq!(g.client_sword_block_flags(), 8);
}
#[test]
fn settings_service_publishes_live_flags_without_borrowing_the_actor() {
    let service = service::SwordBlockService::default();
    assert_eq!(service.flags(), 8);
    let mut s = service.settings();
    s.fake_on_pressing = true;
    s.hide_shield_slot = true;
    s.third_person = false;
    s.always_hide_shield = true;
    s.only_visual = true;
    service.configure(s);
    service.set_running(true);
    assert_eq!(service.flags(), 55);
    assert_eq!(service.settings(), s);
    service.set_running(false);
    assert_eq!(service.flags(), 54);
    assert!(!s.redirect_legacy_use(true, true, true, false));
    s.only_visual = false;
    assert!(!s.redirect_legacy_use(true, true, true, false));
    assert!(s.redirect_legacy_use(false, true, true, false));
}
