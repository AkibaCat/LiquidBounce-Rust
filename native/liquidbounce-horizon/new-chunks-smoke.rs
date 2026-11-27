#[test]
fn new_chunks_original_settings_and_geometry() {
    use crate::new_chunks::NewChunks;
    let mut m = NewChunks::default();
    assert_eq!(
        (m.render_distance, m.render_y, m.auto_y, m.smooth, m.persist),
        (32, 0., false, true, true)
    );
    m.chunk([0, 0]);
    m.flowing([16, 70, 0]);
    m.chunk([1, 0]);
    assert_eq!(m.chunks.get(&[1, 0]), Some(&true));
    let planes = m.render([8., 80., 8.]);
    assert_eq!(planes[0].color, 0x50e91500);
    assert_eq!(planes[0].outline, 0x50a30e00);
    assert_eq!(planes[1].color, 0x5000ff00);
    assert_eq!(
        planes[1].corners,
        [[16., 0., 0.], [16., 0., 16.], [32., 0., 16.], [32., 0., 0.]]
    );
    m.forget([1, 0]);
    assert_eq!(m.chunks.len(), 2);
    m.persist = false;
    m.forget([1, 0]);
    assert_eq!(m.chunks.len(), 1);
    m.auto_y = true;
    m.render_distance = 4;
    assert_eq!(m.render([8., 80., 8.])[0].corners[0][1], -20.);
    assert!(m.render([1000., 80., 1000.]).is_empty());
    m.flowing([-1, 0, -17]);
    assert_eq!(m.chunks.get(&[-1, -2]), Some(&true));
    m.clear();
    assert!(m.chunks.is_empty());
}

#[test]
fn native_packets_runtime_and_geometry() {
    use crate::{packets::play::*, world_primitives::*};
    let game = game::Game::default();
    game.packet(&Clientbound::LevelChunkWithLight(LevelChunkWithLight {
        x: 0,
        z: 0,
    }));
    game.packet(&Clientbound::BlockUpdate(BlockUpdate {
        pos: [1, 70, 1],
        state: 1,
    }));
    assert_eq!(
        game.hooks
            .state()
            .behaviors
            .borrow()
            .new_chunks
            .chunks
            .get(&[0, 0]),
        Some(&false)
    );
    game.packet(&Clientbound::BlockUpdate(BlockUpdate {
        pos: [1, 70, 1],
        state: 2,
    }));
    game.packet(&Clientbound::SectionBlocksUpdate(SectionBlocksUpdate {
        section: [-1, 4, 2],
        changes: vec![(2i64 << 12) | 0xabc],
    }));
    assert_eq!(
        game.hooks
            .state()
            .behaviors
            .borrow()
            .new_chunks
            .chunks
            .get(&[-1, 2]),
        Some(&true)
    );
    let mut out = WorldPrimitives::default();
    game.geometry(&mut out);
    assert_eq!(out.len(), 10);
    assert!(out.iter().all(|p| p.through_walls));
    assert_eq!(
        out.iter()
            .filter(|p| matches!(p.shape, PrimitiveShape::LiquidBounceQuad { .. }))
            .count(),
        2
    );
    assert!(
        out.iter()
            .all(|p| !matches!(p.shape,PrimitiveShape::Line{width,..} if width!=1.))
    );
    game.hooks.state().behaviors.borrow_mut().new_chunks.persist = false;
    game.packet(&Clientbound::ForgetLevelChunk(ForgetLevelChunk {
        pos: [0, 0],
    }));
    assert_eq!(
        game.hooks
            .state()
            .behaviors
            .borrow()
            .new_chunks
            .chunks
            .len(),
        1
    );
    game.hooks.state().behaviors.borrow_mut().running = false;
    game.packet(&Clientbound::BlockUpdate(BlockUpdate {
        pos: [1, 70, 1],
        state: 2,
    }));
    assert_eq!(
        game.hooks
            .state()
            .behaviors
            .borrow()
            .new_chunks
            .chunks
            .len(),
        1
    );
}
