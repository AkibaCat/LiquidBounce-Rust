#[test]
fn placement_renderer_lifecycle_and_culling() {
    use crate::placement_renderer::*;
    let mut r = PlacementRenderer::new(true, 0x5aff0000);
    r.add([0, 0, 0], FULL, 0, false);
    r.add([1, 0, 0], FULL, 0, false);
    r.update_all();
    let (af, ae) = r.cull([0, 0, 0]);
    assert_eq!(af & (15 << 12), 0);
    assert_eq!(ae & (3 << 2), 0);
    let start = r.render(0);
    assert_eq!(start.len(), 2);
    assert_eq!(start[0].color >> 24, 0);
    let transition = r.render(500);
    assert_eq!(transition.len(), 4);
    assert_eq!(transition[0].bounds, FULL);
    r.remove([0, 0, 0], 500);
    assert!(r.contains([0, 0, 0]));
    let mid = r.render(750);
    let fading = mid.iter().find(|d| d.position == [0, 0, 0]).unwrap();
    assert_eq!(fading.color >> 24, 45);
    assert!((fading.bounds[0] - 0.05).abs() < 1e-7);
    r.render(1000);
    assert!(!r.contains([0, 0, 0]));
    assert_ne!(r.cull([1, 0, 0]).0 & (15 << 20), 0);
    r.clear_silently(1000);
    assert!(r.running(false));
    r.render(1500);
    r.tick();
    assert!(!r.running(false));
    let mut transient = PlacementRenderer::new(false, 0x5affb348);
    transient.add([0, 0, 0], FULL, 0, false);
    let first = transient.render(500);
    assert_eq!(first.len(), 2);
    assert_eq!(first[1].bounds, [0.; 6]);
    transient.render(1000);
    assert!(!transient.contains([0, 0, 0]));
}

#[test]
fn void_search_uses_center_support_and_threshold() {
    let scan = |facing| {
        crate::void_search::search([0, 1, 0], facing, 1, 2, 3, -64, |p| {
            p == [0, 0, 0] || p == [0, -1, 1]
        })
    };
    let result = scan([0, 1]);
    assert_eq!(result.len(), 13);
    assert!(!result.contains(&[0, 0, 1]));
    assert!(result.contains(&[1, 0, 2]));
    let turned = scan([1, 0]);
    assert!(turned.contains(&[2, 0, 1]));
    assert!(!turned.contains(&[1, 0, 2]));
    assert!(
        crate::void_search::search([0, 8, 0], [0, 1], 1, 2, 3, -64, |p| p == [0, 0, 0]).is_empty()
    );
}

#[test]
fn native_prophunt_and_void_runtime_smoke() {
    use crate::{game::*, packets::play::*, world_primitives::WorldPrimitives};
    let mut game = Game::default();
    game.world.blocks.insert([0, 0, 0]);
    game.falling.push(Falling {
        position: Position {
            x: 1.8,
            y: 2.1,
            z: -0.1,
        },
    });
    game.tick();
    game.packet(&Clientbound::BlockUpdate(BlockUpdate {
        pos: [2, 2, 2],
        state: 1,
    }));
    game.packet(&Clientbound::SectionBlocksUpdate(SectionBlocksUpdate {
        section: [-1, 0, 0],
        changes: vec![(1 << 12) | 0xabc],
    }));
    {
        let state = game.hooks.state();
        let mut m = state.behaviors.borrow_mut();
        assert!(m.prophunt_esp.render.contains([1, 2, -1]));
        assert!(m.prophunt_esp.render.contains([2, 2, 2]));
        assert!(m.prophunt_esp.render.contains([-6, 12, 11]));
        m.prophunt_esp.render.in_time = 0;
        m.prophunt_esp.render.fade_in_curve = crate::Easing::None;
        m.void_esp.range_facing = 1;
        m.void_esp.range_side = 1;
    }
    game.player_tick();
    assert_eq!(
        game.hooks
            .state()
            .behaviors
            .borrow()
            .void_esp
            .previous
            .len(),
        8
    );
    let mut out = WorldPrimitives::default();
    game.geometry(&mut out);
    assert!(!out.is_empty());
    assert!(out.iter().all(|p| p.through_walls));
    {
        let state = game.hooks.state();
        let mut m = state.behaviors.borrow_mut();
        m.prophunt_running = false;
        m.prophunt_esp.render.clear_silently(0);
    }
    game.packet(&Clientbound::BlockUpdate(BlockUpdate {
        pos: [99, 2, 2],
        state: 1,
    }));
    assert!(
        !game
            .hooks
            .state()
            .behaviors
            .borrow()
            .prophunt_esp
            .render
            .contains([99, 2, 2])
    );
}
