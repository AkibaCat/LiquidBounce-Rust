// SPDX-License-Identifier: GPL-3.0-or-later
#[path = "../liquidbounce-behaviors/src/spear_kill.rs"]
mod spear;
use spear::*;
struct Fixture {
    module: SpearKill,
    using: bool,
    kinetic: bool,
    ticks: i32,
    use_key: bool,
    attack: bool,
    velocities: Vec<Vector>,
    targets: usize,
    restarts: usize,
    predicted: Vector,
}
impl Default for Fixture {
    fn default() -> Self {
        Self {
            module: Default::default(),
            using: true,
            kinetic: true,
            ticks: 12,
            use_key: true,
            attack: true,
            velocities: vec![],
            targets: 0,
            restarts: 0,
            predicted: [0.0, 0.0, 20.0],
        }
    }
}
impl Host for Fixture {
    fn with_spear_kill<R>(&mut self, f: impl FnOnce(&mut SpearKill) -> R) -> R {
        f(&mut self.module)
    }
    fn using_spear(&mut self) -> bool {
        self.using
    }
    fn kinetic(&mut self) -> Option<Kinetic> {
        self.kinetic.then_some(Kinetic {
            delay: 12,
            damage_duration: 24,
        })
    }
    fn ticks_using(&mut self) -> i32 {
        self.ticks
    }
    fn use_down(&mut self) -> bool {
        self.use_key
    }
    fn attack_down(&mut self) -> bool {
        self.attack
    }
    fn restart_using(&mut self) {
        self.restarts += 1;
        self.ticks = 0;
        self.module.settings.max_speed = 2.0;
    }
    fn target(&mut self, _: f32) -> Option<Target> {
        self.targets += 1;
        Some(Target {
            id: 9,
            distance: 14.0,
        })
    }
    fn predict(&mut self, _: i32, ticks: i32) -> Option<Vector> {
        assert!(ticks >= 1);
        Some(self.predicted)
    }
    fn eye(&mut self) -> Vector {
        [0.0; 3]
    }
    fn look(&mut self) -> Vector {
        [0.0, 0.0, 2.0]
    }
    fn velocity(&mut self, v: Vector) {
        self.velocities.push(v);
    }
}
#[test]
fn spear_charge_lunge_and_reset_smoke() {
    let mut f = Fixture::default();
    assert_eq!(f.module.settings, Settings::default());
    SpearKill::tick(true, &mut f);
    assert!(f.velocities.is_empty());
    assert_eq!(f.module.attack_movements.len(), 5);
    assert_eq!(f.targets, 1);
    assert_eq!(
        f.module.current_charge_attack_movement(),
        Some([0.0, 0.0, 7.0])
    );
    f.attack = false;
    for _ in 0..5 {
        SpearKill::tick(true, &mut f);
    }
    assert_eq!(
        f.velocities,
        [
            [0.0, 0.0, 7.0],
            [0.0, 0.0, 7.0],
            [-0.0, -0.0, -7.0],
            [-0.0, -0.0, -7.0],
            [0.0; 3]
        ]
    );
    assert!(f.module.current_charge_attack_movement().is_none());
    f.module.attack_movements.push_back([4.0, 0.0, 0.0]);
    f.ticks = 0;
    SpearKill::tick(true, &mut f);
    assert!(f.module.attack_movements.is_empty());
    assert_eq!(f.velocities.len(), 5);
    f.module.attack_movements.push_back([4.0, 0.0, 0.0]);
    SpearKill::tick(false, &mut f);
    assert_eq!(f.module.attack_movements.len(), 1);
    f.using = false;
    SpearKill::tick(true, &mut f);
    assert!(f.module.attack_movements.is_empty());
    assert_eq!(f.module.preview_target, None);
    assert_eq!(f.velocities.last(), Some(&[0.0; 3]));
    f.using = true;
    f.ticks = 37;
    SpearKill::tick(true, &mut f);
    assert_eq!(f.restarts, 1);
    assert_eq!(f.module.settings.max_speed, 2.0);
    assert!(f.module.attack_movements.is_empty());
    f.module.settings.preview = false;
    f.module.settings.max_speed = 7.0;
    f.ticks = 12;
    f.attack = true;
    f.predicted = [0.0; 3];
    let targets = f.targets;
    SpearKill::tick(true, &mut f);
    assert_eq!(f.targets, targets + 1);
    assert_eq!(f.module.preview_target, Some(9));
    assert_eq!(
        f.module.current_charge_attack_movement(),
        Some([0.0, 0.0, 7.0])
    );
    f.kinetic = false;
    SpearKill::tick(true, &mut f);
    assert_eq!(f.module.preview_target, None);
    assert!(f.module.attack_movements.is_empty());
    assert_eq!(f.velocities.last(), Some(&[0.0; 3]));
    f.kinetic = true;
    f.ticks = 36;
    SpearKill::tick(true, &mut f);
    assert!(f.module.attack_movements.is_empty());
    assert_eq!(f.restarts, 1);
}
