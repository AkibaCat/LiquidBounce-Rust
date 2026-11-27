#[test]
fn source_lifecycle_and_modes() {
    use damage_particles::*;
    let mut s = DamageParticles::default();
    let mut e = Entity {
        identity: 9,
        own: false,
        ticked: true,
        dead: false,
        health: 20.,
        max_health: 20.,
        center: [1., 2., 3.],
        movement: [0.25, 0.5, 0.],
    };
    s.tick(1000, &[e]);
    e.health = 18.75;
    s.tick(1050, &[e]);
    assert_eq!(s.particles().len(), 1);
    assert_eq!(s.particles()[0].text, "1.2");
    assert_eq!(s.particles()[0].position, [1.25, 2.5, 3.]);
    assert_eq!(s.render(1050)[0].scale, 0.);
    let p = s.render(2550).remove(0);
    assert_eq!(p.scale, 1.5);
    assert_eq!(p.position, [1.25, 3.625, 3.]);
    s.tick(4050, &[e]);
    assert_eq!(s.particles().len(), 1);
    s.tick(4051, &[e]);
    assert!(s.particles().is_empty());
    s.track_mode = TrackMode::OnUpdate;
    s.update(4100, e, 18.75, 20., 20.);
    assert_eq!(s.particles()[0].color, s.max_health);
    e.own = true;
    s.update(4101, e, 20., 10., 20.);
    assert_eq!(s.particles().len(), 1);
    s.clear();
    assert!(s.particles().is_empty());
    assert_eq!(number(1.25, "."), "1.2");
    assert_eq!(number(1.75, ","), "1,8");
    assert_eq!(number(2., "."), "2");
}
#[test]
fn actual_runtime_tick_and_overlay() {
    use mod_api::*;
    let hooks = runtime::LiquidBounceHooks::default();
    let mut output = String::new();
    for health in [20., 15.] {
        let input=serde_json::json!({"tick":true,"entities":[{"identity":1,"own":false,"ticked":true,"dead":false,"health":health,"max":20,"center":[0,1,0],"movement":[0,0,0]}]}).to_string();
        hooks.damage_particles(
            &NullHost,
            &mut ClientDamageParticlesEvent {
                input: &input,
                draws: &mut output,
            },
        );
    }
    hooks.damage_particles(
        &NullHost,
        &mut ClientDamageParticlesEvent {
            input: "{}",
            draws: &mut output,
        },
    );
    let v: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(v[0]["text"], "5");
    assert_eq!(v[0]["color"], -65536);
}
#[test]
fn real_setter_capture_does_not_poll_metadata() {
    use damage_particles_capture as c;
    use living_health::{HealthWorld, SyncedFloat};
    struct World(SyncedFloat);
    impl HealthWorld for World {
        type Error = ();
        type Data = ();
        fn health(&mut self) -> Result<f32, ()> {
            Ok(self.0.value())
        }
        fn health_data(&self) {}
        fn max_health_attribute(&mut self) -> Result<f64, ()> {
            Ok(20.)
        }
        fn max_absorption_attribute(&mut self) -> Result<f64, ()> {
            Ok(0.)
        }
        fn store_health(&mut self, _: (), v: f32) -> Result<(), ()> {
            self.0.set(v, false, |_| Ok(()))
        }
        fn store_absorption(&mut self, _: f32) -> Result<(), ()> {
            Ok(())
        }
        fn dead_flag(&self) -> bool {
            false
        }
        fn dispatch_set_health(&mut self, v: f32) -> Result<(), ()> {
            living_health::set_health(self, v)
        }
    }
    let p = motion::PlayerMotion::default();
    c::actor_movement(Some(geometry::Vec3 {
        x: 1.,
        y: 2.,
        z: 3.,
    }));
    let _scope = c::player(&p);
    c::clear();
    let mut world = World(SyncedFloat::new(20.));
    living_health::set_health(&mut world, 15.).unwrap();
    world.0.set(10., false, |_| Ok::<_, ()>(())).unwrap();
    living_health::set_health(&mut world, 8.).unwrap();
    let events = c::drain();
    assert_eq!(events.len(), 2);
    assert_eq!((events[0].old, events[0].new), (20., 15.));
    assert_eq!((events[1].old, events[1].new), (10., 8.));
    assert_eq!(events[0].context.movement, [1., 2., 3.]);
    c::actor_movement(None);
}
