#[cfg(test)]
mod tracers_smoke {
    use super::*;
    use serde_json::{Value, json};
    use tracers::{ColorMode, Entity, Tracers};
    fn entity() -> Entity {
        Entity {
            position: [100., 0., 0.],
            interpolated: [100.25, 0., 0.],
            height: 2.,
            health: 5.,
            max_health: 20.,
            friend: false,
            tag: None,
        }
    }
    #[test]
    fn colors_distance_geometry_and_spline() {
        let mut s = Tracers::default();
        let mut e = entity();
        let lines = s.lines([0.; 3], [0., 0., 1.], &[e], 0.);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0][0].position, [0., 0., 1.]);
        assert_eq!(lines[1][1].position, [100.25, 2., 0.]);
        assert_eq!(lines[0][0].color, [0., 1., 0., 1.]);
        assert!(
            s.lines(
                [0.; 3],
                [0.; 3],
                &[Entity {
                    position: [129., 0., 0.],
                    ..e
                }],
                0.
            )
            .is_empty()
        );
        e.tag = Some(0xff123456);
        assert_eq!(
            s.lines([0.; 3], [0.; 3], &[e], 0.)[0][0].color,
            breadcrumbs::rgba(0xff123456)
        );
        e.friend = true;
        assert_eq!(
            s.lines([0.; 3], [0.; 3], &[e], 0.)[0][0].color,
            [0., 0., 1., 1.]
        );
        e.friend = false;
        e.tag = None;
        s.color_mode = ColorMode::Health;
        assert_eq!(
            s.lines([0.; 3], [0.; 3], &[e], 0.)[0][0].color,
            [191. / 255., 63. / 255., 0., 1.]
        );
        s.color_mode = ColorMode::Rainbow;
        assert_eq!(
            s.lines([0.; 3], [0.; 3], &[e], 0.)[0][0].color,
            [1., 0., 0., 50. / 255.]
        );
        s.render.shader = true;
        assert_eq!(s.lines([0.; 3], [0.; 3], &[e], 0.)[0][0].color[3], 1.);
        for style in &s.render.styles {
            assert_eq!(style.get("Fill"), 1.);
        }
        assert_eq!(tracers::curve::sample(&s.hue, -1.), 0.);
        assert_eq!(tracers::curve::sample(&s.hue, 100.), 120.);
        assert_eq!(tracers::curve::sample(&s.hue, 500.), 120.);
        assert!(tracers::curve::sample(&s.hue, 50.) > 0.);
    }
    #[test]
    fn runtime_tick_cache_combining_friend_and_width() {
        let hooks = runtime::LiquidBounceHooks::default();
        let host = mod_api::NullHost;
        let mut output = String::new();
        let target = json!({"kind":"minecraft:zombie","alive":true,"invisible":false,"sleeping":false,"is_local":false,"has_local_passenger":false,"within_border":true});
        let item = |id: u64| json!({"id":id,"identity":id,"kind":"minecraft:zombie","group":0,"position":[1.,0.,0.],"interpolated":[1.25,0.,0.],"height":2.,"health":20.,"maxHealth":20.,"target":target});
        let mut input = json!({"tick":false,"perspective":0,"camera":[0.,0.,0.],"eye":[0.,0.,1.],"entities":[item(1),item(2)]});
        hooks.tracers(
            &host,
            &mut mod_api::ClientTracersEvent {
                input: &input.to_string(),
                geometry: &mut output,
            },
        );
        assert_eq!(
            serde_json::from_str::<Value>(&output).unwrap()["lines"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        input["tick"] = json!(true);
        hooks.tracers(
            &host,
            &mut mod_api::ClientTracersEvent {
                input: &input.to_string(),
                geometry: &mut output,
            },
        );
        input["tick"] = json!(false);
        hooks.state().behaviors.borrow_mut().tracers.line_width = 4.;
        hooks.tracers(
            &host,
            &mut mod_api::ClientTracersEvent {
                input: &input.to_string(),
                geometry: &mut output,
            },
        );
        let out: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(out["lines"].as_array().unwrap().len(), 2);
        assert_eq!(out["line_width"], 4.);
        assert_eq!(out["lines"][0][1]["position"][0], 1.25);
        hooks.state().behaviors.borrow_mut().tracers.clear();
    }
    #[test]
    fn renderer_expands_physical_width() {
        let p = |x, y| render::ColoredPoint {
            position: [x, y, 0.],
            color: [1.; 4],
        };
        let mut frame = render::Frame {
            quads: vec![],
            lines: vec![[p(-0.5, 0.), p(0.5, 0.)]],
            effect: None,
            line_width: 1.,
        };
        let a = render::vertices(&frame, glam::Mat4::IDENTITY, glam::DVec3::ZERO, [100, 100]);
        frame.line_width = 4.;
        let b = render::vertices(&frame, glam::Mat4::IDENTITY, glam::DVec3::ZERO, [100, 100]);
        assert_eq!(a.len(), 6);
        assert!((b[0].clip[1] / a[0].clip[1] - 4.).abs() < 1e-6);
    }
}
