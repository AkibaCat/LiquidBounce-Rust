#[cfg(test)]
mod smoke {
    use crate::crosshair::{Crosshair, mix};
    use serde_json::{Value, json};
    fn frame(now: i64, charge: Option<f32>, item: f32, attack: f32) -> Value {
        json!({"now":now,"wall":0,"width":320.,"height":180.,"firstPerson":true,"hiddenScreen":false,"charge":charge,"item":item,"attack":attack})
    }
    #[test]
    fn original_modes_dynamic_cooldown_priority_release_flash_and_same_target_damage() {
        let mut s = Crosshair::default();
        let out = s.render(&frame(2000, None, 1., 0.5));
        assert_eq!(out["circle"]["lut"].as_array().unwrap().len(), 256);
        assert_eq!(out["circle"]["inner"], 3.5);
        assert_eq!(out["circle"]["outer"], 5.5);
        s.values.insert("Mode".into(), json!("CS2"));
        let out = s.render(&frame(2000, None, 1., 0.));
        assert_eq!(out["quads"].as_array().unwrap().len(), 4);
        let cy = 180f32 / 2.0025;
        assert!((out["quads"][0][1].as_f64().unwrap() as f32 - (cy - 8.)).abs() < 1e-5);
        s.values.insert("CooldownBar.Enabled".into(), json!(true));
        s.values.insert("HitFlash.Enabled".into(), json!(true));
        let out = s.render(&frame(2000, Some(0.25), 0.5, 0.75));
        assert_eq!(out["quads"].as_array().unwrap().len(), 6);
        let q = &out["quads"][5];
        assert_eq!(q[2].as_f64().unwrap() - q[0].as_f64().unwrap(), 5.);
        let out = s.render(&frame(2010, None, 0.5, 1.));
        let q = &out["quads"][5];
        assert_eq!(q[2].as_f64().unwrap() - q[0].as_f64().unwrap(), 10.);
        assert_eq!(q[4], -1);
        let out = s.render(&frame(2020, None, 1., 1.));
        assert_eq!(out["quads"][5][4], 0xff4677ffu32 as i32);
        assert_eq!(
            out["quads"][5][2].as_f64().unwrap() - out["quads"][5][0].as_f64().unwrap(),
            20.
        );
        assert_eq!(
            s.render(&frame(2271, None, 1., 1.))["quads"]
                .as_array()
                .unwrap()
                .len(),
            4
        );
        s.tick(3000, Some("first".into()), 0, true);
        s.tick(3010, Some("first".into()), 10, true);
        let out = s.render(&frame(3010, None, 1., 1.));
        assert_eq!(out["quads"][0][4], 0xffff2828u32 as i32);
        s.tick(3020, Some("second".into()), 10, true);
        let out = s.render(&frame(3160, None, 1., 1.));
        assert_eq!(
            out["quads"][0][4],
            mix(0xff4677ffu32 as i32, 0xffff2828u32 as i32, 0.5)
        );
        s.set("HitFlash.Enabled".into(), json!(false));
        s.set("HitFlash.Enabled".into(), json!(true));
        assert_eq!(
            s.render(&frame(3160, None, 1., 1.))["quads"][0][4],
            0xff4677ffu32 as i32
        );
        s.render(&frame(3170, Some(0.5), 1., 1.));
        s.set("CooldownBar.Enabled".into(), json!(false));
        s.set("CooldownBar.Enabled".into(), json!(true));
        assert_eq!(
            s.render(&frame(3180, Some(1.), 1., 1.))["quads"]
                .as_array()
                .unwrap()
                .len(),
            4
        );
        s.tick(3400, None, 0, false);
        assert_eq!(s.state, Default::default());
        s.values.insert("Mode.CS2.Color.Sync".into(), json!(false));
        s.values
            .insert("Mode.CS2.Color.SpinSpeed".into(), json!(0.));
        s.values.insert("HitFlash.Enabled".into(), json!(false));
        let out = s.render(&frame(4000, None, 1., 1.));
        assert_eq!(
            out["quads"][0][4],
            mix(
                0xff4677ffu32 as i32,
                mix(0xff4677ffu32 as i32, -1, 0.5),
                0.5
            )
        );
    }
}
