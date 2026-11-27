#[test]
fn connected_hit_face_and_full_surface_are_real_union_meshes() {
    use block_outline::{BlockOutline, Target};
    let boxes = [
        [0., 0., 0., 1., 0.5, 1.],
        [0., 0.5, 0., 0.5, 1., 1.],
        [2., 0., 0., 3., 1., 1.],
    ];
    let mut s = BlockOutline::default();
    let mesh = s.render(
        Some(Target {
            boxes: &boxes,
            edges: &[],
            side: 2,
            hit: [0.25, 0.75, 0.],
        }),
        0,
    );
    assert!(!mesh.quads.is_empty());
    assert!(mesh.quads.iter().flatten().all(|p| p[0] <= 1.));
    assert_eq!(mesh.quads.len(), 2);
    assert_eq!(mesh.lines.len(), 6);
    s.side_only = false;
    let mesh = s.render(
        Some(Target {
            boxes: &boxes,
            edges: &[[[0., 0., 0.], [1., 0., 0.]]],
            side: 2,
            hit: [0.25, 0.75, 0.],
        }),
        0,
    );
    assert_eq!(mesh.lines.len(), 1);
    assert!(mesh.quads.len() > 6);
}
#[test]
fn single_box_slides_then_missing_target_resets() {
    use block_outline::{BlockOutline, Target};
    let mut s = BlockOutline::default();
    let a = [[0., 0., 0., 1., 1., 1.]];
    let b = [[2., 0., 0., 3., 1., 1.]];
    let target = |boxes| {
        Some(Target {
            boxes,
            edges: &[],
            side: 1,
            hit: [0.5, 1., 0.5],
        })
    };
    assert_eq!(s.render(target(&a), 0).quads[0][0][0], 0.);
    assert_eq!(s.render(target(&b), 100).quads[0][0][0], 0.);
    assert_eq!(s.render(target(&b), 175).quads[0][0][0], 1.);
    assert!(s.render(None, 200).quads.is_empty());
    assert_eq!(s.render(target(&b), 201).quads[0][0][0], 2.);
}
#[test]
fn runtime_serializes_actual_mesh_and_query_keeps_animation() {
    let hooks = runtime::LiquidBounceHooks::default();
    let mut active = false;
    let mut geometry = String::new();
    let input = r#"{"boxes":[[0,0,0,1,1,1]],"edges":[],"hit":[0.5,1,0.5],"side":1}"#;
    let mut event = mod_api::ClientBlockOutlineEvent {
        input,
        geometry: &mut geometry,
        active,
    };
    hooks.block_outline(&mod_api::NullHost, &mut event);
    active = event.active;
    assert!(active);
    let data: serde_json::Value = serde_json::from_str(&geometry).unwrap();
    assert_eq!(data["quads"].as_array().unwrap().len(), 1);
    assert_eq!(data["lines"].as_array().unwrap().len(), 4);
}
