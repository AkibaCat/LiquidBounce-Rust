// SPDX-License-Identifier: GPL-3.0-or-later
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    #[test]
    fn complete_settings_and_inactive_timer_roundtrip() {
        use config::*;
        let mut s = tnt_timer::TNTTimer::default();
        for (p, v) in [
            ("ESP", json!(false)),
            (
                "ShowTimer.Scale",
                json!([{"x":0.,"y":0.25},{"x":50.,"y":3.},{"x":200.,"y":4.}]),
            ),
            ("ShowTimer.RenderY", json!(-1.5)),
            ("ShowTimer.TimeUnit", json!("Seconds")),
            ("ShowTimer.Show", json!(["Offscreen"])),
        ] {
            tnt_timer::TNTTimerSetting(p.into(), v).apply(&mut s);
        }
        let common = CommonSettings {
            enabled: true,
            hidden: false,
            key_bind: Value::Null,
        };
        for format in [JsonFormat::File, JsonFormat::Interop] {
            let tree = serialize_tnt_timer(&s, &common, format, &str::to_owned).unwrap();
            let (changes, errors) = parse_tnt_timer_value(&tree, "TNTTimer");
            assert!(errors.is_empty(), "{errors:?}");
            assert_eq!(changes.len(), s.values.len());
            let mut restored = tnt_timer::TNTTimer::default();
            for change in changes {
                change.apply(&mut restored);
            }
            assert_eq!(
                serialize_tnt_timer(&restored, &common, format, &str::to_owned).unwrap(),
                tree
            );
            if format == JsonFormat::Interop {
                assert_eq!(tree["value"][3]["value"][1]["xAxis"]["label"], "Distance");
            }
        }
        let tree = json!({"value":[{"name":"ShowTimer","value":[{"name":"Show","value":[]}]}]});
        assert!(!parse_tnt_timer_value(&tree, "TNTTimer").1.is_empty());
    }
    #[test]
    fn countdown_uses_current_center_fuse_priority_curve_and_owner() {
        let mut s = tnt_timer::TNTTimer::default();
        tnt_timer::TNTTimerSetting("ShowTimer.Enabled".into(), json!(true)).apply(&mut s);
        tnt_timer::TNTTimerSetting("ShowTimer.TimeUnit".into(), json!("Seconds")).apply(&mut s);
        let owner = json!({"text":"Owner","color":"green"});
        let input = json!({"camera":[0.,0.,0.],"entities":[{"id":1,"center":[0.,2.,10.],"fuse":25,"owner":owner},{"id":2,"center":[0.,0.,0.],"fuse":0}]});
        let output = s.frame(&input);
        let labels = output["labels"].as_array().unwrap();
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0]["position"], json!([0., 3., 10.]));
        assert_eq!(labels[0]["priority"], -25);
        assert_eq!(labels[0]["text"], "1.2s");
        assert_eq!(labels[0]["owner"], owner);
        assert_eq!(labels[0]["scale"], 1.);
        assert_eq!(tnt_timer::TNTTimer::countdown_color(80), 0xffffff00);
        assert_eq!(tnt_timer::TNTTimer::countdown_color(1), 0xffff0300);
        for fuse in [1, 20, 79, 80, 400] {
            let red = (255. * (1. + 0.5 * (2400f64 / (12 + fuse) as f64).sin())).floor() as i32;
            assert_eq!(
                tnt_timer::TNTTimer::glow(fuse),
                0xff000000 | ((red.clamp(0, 255) as u32) << 16)
            );
        }
    }
    #[test]
    fn seconds_match_original_decimal_format_oracle() {
        let vectors: std::collections::BTreeMap<String, String> =
            serde_json::from_str(include_str!("tnt-timer-time-vectors.json")).unwrap();
        assert_eq!(vectors.len(), 4101);
        let mut settings = tnt_timer::TNTTimer::default();
        tnt_timer::TNTTimerSetting("ShowTimer.TimeUnit".into(), json!("Seconds"))
            .apply(&mut settings);
        for (ticks, expected) in vectors {
            let ticks = ticks.parse().unwrap();
            assert_eq!(settings.time(ticks), expected, "ticks={ticks}");
        }
    }
    #[test]
    fn actual_runtime_filters_disabled_esp_and_module() {
        use mod_api::ClientTntTimerEvent;
        let h = runtime::LiquidBounceHooks::default();
        let mut output = String::new();
        h.tnt_timer(
            &(),
            &mut ClientTntTimerEvent {
                input: r#"{"action":"glow","fuse":20}"#,
                output: &mut output,
            },
        );
        assert!(serde_json::from_str::<u32>(&output).is_ok());
        h.state()
            .behaviors
            .borrow_mut()
            .tnt_timer
            .values
            .insert("ESP".into(), json!(false));
        output.clear();
        h.tnt_timer(
            &(),
            &mut ClientTntTimerEvent {
                input: r#"{"action":"glow","fuse":20}"#,
                output: &mut output,
            },
        );
        assert!(output.is_empty());
        h.state().behaviors.borrow_mut().active = false;
        h.tnt_timer(
            &(),
            &mut ClientTntTimerEvent {
                input: r#"{"action":"frame","camera":[0,0,0],"entities":[]}"#,
                output: &mut output,
            },
        );
        assert!(output.is_empty());
    }
    #[test]
    fn indicator_original_viewport_inset_behind_and_zero_direction() {
        use indicator::*;
        let p = project([0., 0., 0., 1.], [800., 600.], 2.);
        let l = layout(
            p,
            [400., 300.],
            [-10., -5., 10., 5.],
            0.,
            [true, true],
            10.,
            0.,
        )
        .unwrap();
        assert!(l.in_view);
        assert_eq!(l.anchor, [200., 150.]);
        assert_eq!(l.tag, [200., 143.]);
        assert_eq!(l.length, 0.);
        let p = project([2., 0., 0., -1.], [800., 600.], 2.);
        let l = layout(
            p,
            [400., 300.],
            [-10., -5., 10., 5.],
            0.,
            [true, true],
            10.,
            0.,
        )
        .unwrap();
        assert!(!l.in_view);
        assert_eq!(l.anchor, [399., 150.]);
        assert_eq!(l.tag, [377., 150.]);
        assert!(layout(p, [400., 300.], [0.; 4], 0., [true, false], 10., 0.).is_none());
        assert!(
            layout(
                project([0., 0., 0., -1.], [800., 600.], 2.),
                [400., 300.],
                [0.; 4],
                0.,
                [true, true],
                10.,
                0.
            )
            .is_none()
        );
        let l = layout(
            project([0.9, 0., 0., 1.], [800., 600.], 2.),
            [400., 300.],
            [0.; 4],
            0.9,
            [true, true],
            10.,
            0.,
        )
        .unwrap();
        assert!(l.in_view);
        assert!((l.anchor[0] - 219.).abs() < 0.001);
        assert_eq!(fit_inside(-20., 500., 400.), 20.);
        assert_eq!(border_point(0., 0., 200., 150., 1.), [0., 0.]);
    }
    #[test]
    fn new_arrow_sdf_vertex_layout_and_fixed_point_quantization() {
        let module = naga::front::wgsl::parse_str(include_str!("indicator.wgsl")).unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap();
        assert_eq!(std::mem::size_of::<render::Vertex>(), 44);
        render::clear();
        render::submit([100., 50.], 0., [8.02, 10.], 0., 0xffff0000, [200., 100.]);
        let vertices = render::pending_vertices();
        assert_eq!(vertices.len(), 6);
        assert_eq!(vertices[0].size, [8.03125, 10.]);
        assert_eq!(vertices[0].local, [-4.01, 0.]);
        assert_eq!(vertices[0].color, [1., 0., 0., 1.]);
    }
}
