
#[test]
fn original_modules_drive_typed_packets_and_exact_entity_setters() {
    use std::sync::Arc;
    use packet_modules::{MoreCarry, MoreCarryListener, NoPitchLimit, NoPitchLimitListener};
    use packets::PacketPipeline;

    let pipeline = Arc::new(PacketPipeline::default());
    let carry = Arc::new(MoreCarry::default());
    let pitch = Arc::new(NoPitchLimit::default());
    pipeline.add_listener(0, Arc::new(MoreCarryListener(carry.clone())));
    pipeline.add_listener(0, Arc::new(NoPitchLimitListener(pitch.clone())));
    pipeline.install();

    let mut close = S::ContainerClose(play::ContainerClose { container: 0 });
    assert!(!packet_bridge::outgoing(17, &mut close, false));
    carry.set_running(true);
    for container in [0, 42, -1] {
        let mut close = S::ContainerClose(play::ContainerClose { container });
        assert!(packet_bridge::outgoing(17, &mut close, false));
        // sendPacketSilently bypasses ordinary handlers even with the module on.
        assert!(!packet_bridge::outgoing(17, &mut close, true));
    }
    carry.set_running(false);
    assert!(!packet_bridge::outgoing(17, &mut close, false));

    // The unmodified pinned Entity.turn wraps setXRot, then clamps it and xRotO.
    let normal = turn_rotation(5.0, 80.0, Some(75.0), 20.0, 100.0);
    assert_eq!((normal.y_rot, normal.x_rot, normal.x_rot_o), (8.0, 90.0, 90.0));
    assert_eq!(rope_old_after_turn((4.0, 70.0), 5.0, 80.0, 20.0, 100.0), (7.0, 85.0));
    assert_eq!(packet_bridge::set_x_rot(450.0), 90.0);

    pitch.set_running(true);
    let unlimited = turn_rotation(5.0, 350.0, Some(340.0), 20.0, 200.0);
    assert_eq!((unlimited.y_rot, unlimited.x_rot, unlimited.x_rot_o), (8.0, 380.0, 370.0));
    assert_eq!(rope_old_after_turn((4.0, 340.0), 5.0, 350.0, 20.0, 200.0), (7.0, 370.0));
    assert_eq!(packet_bridge::set_x_rot(-450.0), -450.0);
    assert_eq!(packet_bridge::pitch_limit(225.0, -90.0, 90.0), 225.0);
    let mut mob = Mob::default();
    mob.set_rotation([12.0, 225.0]);
    assert_eq!((mob.yaw, mob.head_pitch, mob.previous_head_pitch), (12.0, 225.0, 225.0));

    let mut rotation = S::MovePlayerRot(play::MovePlayerRot { yaw: 1.0, pitch: 225.0, flags: 3 });
    assert!(!packet_bridge::outgoing(17, &mut rotation, false));
    // Default ServerSide=true preserves the out-of-range pitch on the wire.
    let wire = rotation.encode();
    let mut reader = horizon_protocol::buf::Reader::new(&wire);
    let id = reader.varint().unwrap();
    assert_eq!(S::decode(id, reader.rest()).unwrap(), rotation);
    assert!(matches!(rotation, S::MovePlayerRot(play::MovePlayerRot { pitch: 225.0, flags: 3, .. })));
    pitch.set_server_side(false);
    assert!(!packet_bridge::outgoing(17, &mut rotation, false));
    assert!(matches!(rotation, S::MovePlayerRot(play::MovePlayerRot { pitch: 90.0, flags: 3, .. })));
    let mut movement = S::MovePlayerPosRot(play::MovePlayerPosRot { x: 1.0, y: 2.0, z: 3.0, yaw: 4.0, pitch: -225.0, flags: 2 });
    packet_bridge::outgoing(17, &mut movement, false);
    assert!(matches!(movement, S::MovePlayerPosRot(play::MovePlayerPosRot { pitch: -90.0, yaw: 4.0, flags: 2, x: 1.0, y: 2.0, z: 3.0 })));
    let mut silent = S::MovePlayerRot(play::MovePlayerRot { yaw: 1.0, pitch: 225.0, flags: 0 });
    packet_bridge::outgoing(17, &mut silent, true);
    assert!(matches!(silent, S::MovePlayerRot(play::MovePlayerRot { pitch: 225.0, .. })));

    let mut game = Game { window: true, player: Some(()), yaw: 5.0, pitch: 350.0,
        player_model: Some(Model { previous_head_pitch: 340.0, head_pitch: 350.0 }), sent: Vec::new() };
    game.remote_rotation(&play::PlayerRotation { yaw: 3.0, pitch: 30.0, relative_yaw: true, relative_pitch: true });
    assert_eq!((game.yaw, game.pitch), (8.0, 380.0));
    assert_eq!(game.player_model.as_ref().unwrap().previous_head_pitch, 380.0);
    assert!(matches!(game.sent.pop(), Some(S::MovePlayerRot(play::MovePlayerRot { pitch: 380.0, flags: 0, .. }))));

    pitch.set_running(false);
    assert!(game.native_client_reset_pitch());
    assert_eq!(game.pitch, 90.0);
    assert_eq!(game.player_model.as_ref().unwrap().head_pitch, 90.0);
    // onDisabled sets xRot only, preserving the original old-frame pitch.
    assert_eq!(game.player_model.as_ref().unwrap().previous_head_pitch, 380.0);
    assert!(game.sent.is_empty());
    game.pitch = -450.0;
    assert!(game.native_client_reset_pitch());
    assert_eq!(game.pitch, -90.0);
    game.player = None;
    game.pitch = 200.0;
    assert!(!game.native_client_reset_pitch());
    assert_eq!(game.pitch, 200.0);
    mob.set_rotation([12.0, 225.0]);
    assert_eq!(mob.head_pitch, 90.0);
    assert_eq!(packet_bridge::set_x_rot(450.0), 90.0);
    packet_bridge::set_interceptor(None);
}
