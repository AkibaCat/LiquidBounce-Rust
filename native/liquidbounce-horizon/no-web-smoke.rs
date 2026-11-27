// SPDX-License-Identifier: GPL-3.0-or-later
#[path = "../liquidbounce-behaviors/src/no_web.rs"]
mod no_web;
use no_web::*;
use std::collections::BTreeMap;
#[derive(Default)]
struct Level(BTreeMap<Pos, Block>);
impl World for Level {
    fn block(&self, p: Pos) -> Block {
        self.0.get(&p).cloned().unwrap_or(Block {
            air: true,
            replaceable: true,
            ..Default::default()
        })
    }
}
#[test]
fn five_modes_and_actual_water_action_boundaries() {
    let mut n = NoWeb::default();
    assert!(!n.collision(false, [0; 3], true, true, 0).cancel);
    assert!(n.collision(true, [0; 3], true, true, 0).cancel);
    n.settings.mode = Mode::Grim;
    let e = n.collision(true, [0; 3], false, false, 0);
    assert!(e.cancel && e.break_world && e.destroy_packets);
    n.settings.break_on_world = false;
    assert!(!n.collision(true, [0; 3], false, false, 0).break_world);
    n.settings.mode = Mode::Intave;
    let e = n.collision(true, [0; 3], true, true, 3);
    assert!(!e.cancel && !e.jump);
    assert_eq!(e.blend, 0.734);
    assert!(n.collision(true, [0; 3], true, true, 4).jump);
    assert_eq!(n.collision(true, [0; 3], true, true, 4).blend, 0.346);
    assert_eq!(n.collision(true, [0; 3], true, false, 4).speed, None);
    n.settings.mode = Mode::Strafe;
    n.settings.only_on_ground = true;
    n.settings.motion_y = true;
    let e = n.collision(true, [0; 3], true, false, 0);
    assert_eq!(e.speed, None);
    assert_eq!(e.y, Some(0.6f32 as f64));
    n.settings.only_on_ground = false;
    assert_eq!(
        n.collision(true, [0; 3], true, false, 0).speed,
        Some(0.23f32 as f64)
    );
    n.settings.mode = Mode::PlaceWater;
    for x in 0..10 {
        n.collision(true, [x, 0, 0], false, false, 0);
    }
    assert_eq!(n.webs.front(), Some(&[2, 0, 0]));
    assert_eq!(n.webs.len(), 8);
    n.collision(true, [2, 0, 0], false, false, 0);
    assert_eq!(n.webs.front(), Some(&[2, 0, 0]));
    n.reset();
    let web = [0; 3];
    let mut w = Level::default();
    w.0.insert(
        web,
        Block {
            web: true,
            outline: vec![[0., 0., 0., 1., 1., 1.]],
            ..Default::default()
        },
    );
    n.collision(true, web, false, false, 0);
    let mut p = Player {
        eyes: [0.5, 2., 0.5],
        look: [0., 0., 1.],
        reach: 4.5,
        evaporates: false,
        water_slot: Some(2),
        bucket_slot: Some(40),
    };
    let top = n.plan(&w, p, 1000).unwrap();
    assert!(matches!(
        top.kind,
        ActionKind::Top {
            target: [0, 1, 0],
            ..
        }
    ));
    assert_eq!(top.hit.face, FACES[1]);
    assert_eq!(top.resolve(None, None), None);
    let mut ray = top.hit;
    ray.point[1] = -0.01;
    assert_eq!(top.resolve(Some(ray), None), None);
    assert_eq!(top.resolve(Some(top.hit), None), Some(top.hit));
    n.success(top, top.hit, 1000, &w);
    assert!(n.webs.is_empty());
    n.collision(true, web, false, false, 0);
    assert!(n.plan(&w, p, 1250).is_none());
    assert!(n.plan(&w, p, 1251).is_some());
    n.last_success = None;
    p.eyes = [0.5, 0.5, 0.5];
    let directional = n.plan(&w, p, 1300).unwrap();
    assert!(matches!(
        directional.kind,
        ActionKind::Directional {
            side: [0, 0, 1],
            ..
        }
    ));
    let opposite = Hit {
        pos: [0, 0, 1],
        face: [0, 0, -1],
        point: [0.5, 0.5, 1.],
    };
    assert_eq!(directional.resolve(Some(opposite), None), Some(opposite));
    assert_eq!(directional.resolve(None, None), Some(directional.hit));
    n.success(directional, opposite, 1300, &w);
    assert_eq!(n.pickups.back().unwrap().0, [1, 0, 0]);
    n.webs.clear();
    n.pickups.clear();
    w.0.insert(
        [0, 0, 1],
        Block {
            water_source: true,
            ..Default::default()
        },
    );
    n.record([0, 0, 1], 1000);
    n.record([0, 0, 1], 1000);
    assert!(n.plan(&w, p, 1800).is_none());
    let pickup = n.plan(&w, p, 1801).unwrap();
    assert_eq!(pickup.slot, 40);
    let fluid = Hit {
        pos: [0, 0, 1],
        face: FACES[2],
        point: [0.5, 0.4, 1.],
    };
    assert_eq!(pickup.resolve(None, Some(fluid)), Some(fluid));
    n.success(pickup, fluid, 1801, &w);
    assert!(n.pickups.is_empty());
    for i in 0..20 {
        n.record([0, 0, 1], i);
    }
    assert_eq!(n.pickups.len(), 16);
    assert_eq!(n.pickups.front().unwrap().1, 4);
    p.evaporates = true;
    assert!(n.plan(&w, p, 2000).is_none());
    assert_eq!(Mode::parse("Grim2365"), Some(Mode::Grim));
    assert_eq!(Mode::parse("Intave14"), Some(Mode::Intave));
}
