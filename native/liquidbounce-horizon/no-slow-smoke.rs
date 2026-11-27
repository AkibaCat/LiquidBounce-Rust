// SPDX-License-Identifier: GPL-3.0-or-later
#[path = "../liquidbounce-behaviors/src/no_slow.rs"]
mod model;
#[path = "no-slow-policy.rs"]
mod policy;
use model::*;
#[test]
fn no_slow_all_groups_modes_and_packet_boundaries() {
    let mut n = NoSlow::default();
    let mut c = Context {
        using: true,
        consumable: true,
        action: Action::Eat,
        moving: true,
        selected: 8,
        ..Default::default()
    };
    assert_eq!(n.use_multiplier(true, c, [0.2, 0.2]), [1., 1.]);
    assert_eq!(n.use_multiplier(false, c, [0.2, 0.2]), [0.2, 0.2]);
    for action in [
        Action::Eat,
        Action::Drink,
        Action::Block,
        Action::Spyglass,
        Action::Horn,
        Action::Brush,
        Action::Bow,
        Action::Trident,
        Action::Crossbow,
        Action::Bundle,
        Action::Spear,
    ] {
        c.action = action;
        assert_eq!(n.use_multiplier(true, c, [0.2, 0.2]), [1., 1.]);
    }
    c.action = Action::None;
    assert_eq!(n.use_multiplier(true, c, [0.3, 0.4]), [0.3, 0.4]);
    c.action = Action::Eat;
    n.settings.uses[1].forward = 0.7;
    n.settings.uses[1].sideways = 0.;
    assert_eq!(n.use_multiplier(true, c, [0.2, 0.2]), [0.7, 0.]);
    n.prevent = true;
    assert_eq!(n.use_multiplier(true, c, [0.2, 0.2]), [0.2, 0.2]);
    n.prevent = false;
    n.settings.uses[1].mode = Mode::Grim2360;
    let commands = n.network(true, c, true);
    assert_eq!(
        commands,
        vec![Send {
            command: Command::Use {
                hand: 1,
                predict: false
            },
            untracked: true
        }]
    );
    assert!(n.network(true, c, false).is_empty());
    c.hand = 1;
    assert_eq!(
        n.network(true, c, true)
            .iter()
            .map(|s| s.command)
            .collect::<Vec<_>>(),
        vec![
            Command::Carried(1),
            Command::Carried(3),
            Command::Carried(8)
        ]
    );
    n.settings.uses[1].mode = Mode::Grim2364;
    c.hand = 0;
    assert_eq!(n.network(true, c, true).len(), 3);
    n.settings.uses[1].mode = Mode::Intave14;
    c.ticks = 3;
    assert_eq!(
        n.network(true, c, true)
            .iter()
            .map(|s| s.command)
            .collect::<Vec<_>>(),
        vec![
            Command::Release { at_player: true },
            Command::StopUsing,
            Command::Release { at_player: true }
        ]
    );
    n.settings.intave_new = true;
    c.remaining = 100;
    assert!(n.network(true, c, true).is_empty());
    c.ticks = 2;
    assert_eq!(n.network(true, c, true).len(), 1);
    n.settings.uses[1].mode = Mode::Release;
    c.moving = false;
    assert!(n.network(true, c, true).is_empty());
    c.moving = true;
    assert_eq!(n.network(true, c, true).len(), 1);
    n.settings.uses[1].mode = Mode::Jump;
    assert!(n.jump(true, c));
    c.consumable = false;
    assert!(!n.jump(true, c));
    c.consumable = true;
    n.settings.uses[1].mode = Mode::None;
    c.action = Action::Block;
    c.blocking = true;
    c.tracked_hand = Some(1);
    n.settings.only_slow_server = true;
    assert_eq!(n.use_multiplier(true, c, [0.2, 0.2]), [0.2, 0.2]);
    n.settings.only_slow_server = false;
    n.settings.uses[0].mode = Mode::Reuse;
    assert_eq!(n.network(true, c, true).len(), 1);
    assert_eq!(n.network(true, c, false).len(), 2);
    n.settings.reuse_timing = Timing::Pre;
    assert_eq!(n.network(true, c, true).len(), 2);
    assert_eq!(n.network(true, c, false).len(), 1);
    n.settings.uses[0].mode = Mode::Switch;
    assert_eq!(n.network(true, c, true)[0].command, Command::Carried(1));
    assert_eq!(
        n.network(true, c, false)
            .iter()
            .map(|s| s.command)
            .collect::<Vec<_>>(),
        vec![
            Command::Carried(8),
            Command::Use {
                hand: 1,
                predict: true
            }
        ]
    );
    n.settings.uses[0].mode = Mode::Interact;
    assert!(n.network(true, c, true).is_empty());
    assert_eq!(n.network(true, c, false).len(), 2);
    n.settings.uses[0].mode = Mode::None;
    c.sneaking = true;
    n.settings.sneak_mode = Mode::Switch;
    assert_eq!(
        n.network(true, c, true)[0].command,
        Command::LegacySneak(true)
    );
    assert_eq!(
        n.network(true, c, false)[0].command,
        Command::LegacySneak(false)
    );
    n.settings.sneak_mode = Mode::Aac5;
    assert_eq!(
        n.network(true, c, false)[0].command,
        Command::LegacySneak(true)
    );
    for g in [0, 1, 2, 4] {
        for mode in modes(g) {
            assert_eq!(Mode::parse(mode.tag()), Some(*mode));
        }
    }
    assert_eq!(Mode::parse("Grim2364-1.8"), Some(Mode::Grim2364));
    assert_eq!(Mode::parse("AAC5"), Some(Mode::Aac5));
    assert_eq!(Timing::parse("Post"), Some(Timing::Post));
    policy::configure(n.policy(true, c));
    assert!(!policy::running());
    assert_eq!(policy::factor(2), None);
    assert!(policy::fluid_push());
    let _owner = policy::owner_scope(true);
    assert!(policy::running());
    {
        let _server = policy::owner_scope(false);
        assert!(!policy::running());
        assert_eq!(policy::factor(2), None);
        assert!(policy::fluid_push());
    }
    assert!(policy::running());
    assert_eq!(policy::factor(2), Some(1.));
    assert_eq!(policy::block_factor("minecraft:soul_sand", 0.4), 1.);
    assert_eq!(policy::friction("minecraft:slime_block", 0.8), 0.6);
    assert!(!policy::fluid_push());
    assert!(policy::fluid_collision());
    n.settings.fluid_collision = false;
    policy::configure(n.policy(true, c));
    assert!(!policy::fluid_collision());
    policy::configure(n.policy(false, c));
    assert_eq!(policy::factor(3), None);
    assert!(policy::fluid_push());
    assert!(policy::fluid_collision());
}
