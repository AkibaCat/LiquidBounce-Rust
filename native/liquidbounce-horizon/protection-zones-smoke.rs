#[test]
fn protection_zones_original_geometry_and_indicator() {
    use crate::protection_zones::ProtectionZones;
    let mut s = ProtectionZones::default();
    assert_eq!(
        (
            s.radius_x,
            s.radius_z,
            s.radius_y,
            s.render_limit,
            s.hold_block_to_render,
            s.snap_y
        ),
        (20, 20, 383, 16, false, false)
    );
    let d = s.render(&[[0, 64, 0]], [22., 64., 0.], -64, 319, false);
    assert_eq!(d.len(), 3);
    assert_eq!(d[0].bounds, [-20., -64., -20., 21., 320., 21.]);
    assert_eq!(d[2].fill, 0x3300ff00);
    assert_eq!(
        s.render(&[[0, 64, 0]], [0., 64., 0.], -64, 319, false)
            .len(),
        2
    );
    let d = s.render(&[[0, 64, 0]], [41., 90., 0.], -64, 319, true);
    assert_eq!(d.last().unwrap().bounds, [41., 90., 0., 42., 91., 1.]);
    assert_eq!(d.last().unwrap().fill, 0x33fff000);
    s.snap_y = true;
    assert_eq!(
        s.render(&[[0, 64, 0]], [41., 90., 0.], -64, 319, true)
            .last()
            .unwrap()
            .bounds[1],
        64.
    );
    s.hold_block_to_render = true;
    assert!(
        s.render(&[[0, 64, 0]], [41., 90., 0.], -64, 319, false)
            .is_empty()
    );
    s.hold_block_to_render = false;
    s.render_limit = 1;
    assert_eq!(
        s.render(&[[100, 0, 0], [0, 100, 0]], [0., 0., 0.], -64, 319, false)[1].bounds,
        [0., 100., 0., 1., 101., 1.]
    );
    // Radius1 => grid step3; -2/3 rounds to -1 (Kotlin half-up convention).
    s.radius_x = 1;
    s.radius_z = 1;
    assert_eq!(
        s.render(&[[0, 64, 0]], [-2., 70., 0.], -64, 319, true)
            .last()
            .unwrap()
            .bounds[0],
        -3.
    );
}
#[test]
fn native_actual_runtime_sections_edits_unloads_and_settings() {
    use crate::{game::*, world_primitives::PrimitiveShape};
    let mut g = Game::default();
    g.insert(0, 0, 0, 1);
    let first = g.render();
    assert_eq!(first.iter().count(), 30);
    assert_eq!(
        first
            .iter()
            .filter(|p| matches!(p.shape, PrimitiveShape::LiquidBounceQuad { .. }))
            .count(),
        6
    );
    assert_eq!(g.render().iter().count(), 30);
    g.set(0, 0, 0, 0);
    assert_eq!(g.render().iter().count(), 0);
    g.set(0, 0, 0, 2);
    assert_eq!(g.render().iter().count(), 0);
    g.hooks
        .state()
        .behaviors
        .borrow_mut()
        .protection_zones
        .protection_blocks = vec!["minecraft:diamond_block".into()];
    assert_eq!(g.render().iter().count(), 30);
    g.world.chunks.clear();
    assert_eq!(g.render().iter().count(), 0);
    g.insert(-1, -2, 4095, 2);
    g.player.as_mut().unwrap().position = Position {
        x: 41.,
        y: 70.,
        z: 0.,
    };
    g.offhand = Some(Stack {
        item: "minecraft:diamond_block".into(),
    });
    let rendered = g.render();
    assert_eq!(rendered.iter().count(), 42);
    assert!(rendered.iter().all(|p| p.through_walls));
    g.hooks.state().behaviors.borrow_mut().running = false;
    assert_eq!(g.render().iter().count(), 0);
    g.hooks.state().behaviors.borrow_mut().running = true;
    assert_eq!(g.render().iter().count(), 42);
}
