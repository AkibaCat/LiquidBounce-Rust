#![allow(dead_code)]
#[path = "../liquidbounce-behaviors/src/auto_bow_math.rs"]
mod auto_bow_math;
#[path = "../liquidbounce-behaviors/src/auto_pearl.rs"]
mod auto_pearl;
#[path = "../liquidbounce-behaviors/src/rotations.rs"]
mod rotations;
use auto_bow_math::{B, V};
use auto_pearl::*;
use rotations::Rotation;
fn main() {
    let mut pearl = AutoPearl::default();
    let spawn = Spawn {
        owner: None,
        owner_self: false,
        owner_attackable: false,
        aura_target: None,
        crosshair_cos: 1.,
    };
    assert!(pearl.can_trigger(spawn));
    pearl.settings.mode = Mode::Target;
    assert!(!pearl.can_trigger(spawn));
    assert!(pearl.can_trigger(Spawn {
        owner: Some(8),
        aura_target: Some(8),
        ..spawn
    }));
    assert!(!pearl.can_trigger(Spawn {
        owner: Some(8),
        aura_target: Some(8),
        owner_self: true,
        ..spawn
    }));
    pearl.settings.mode = Mode::Trigger;
    assert!(!pearl.can_trigger(Spawn {
        owner: Some(8),
        ..spawn
    }));
    assert!(pearl.can_trigger(Spawn {
        owner: Some(8),
        owner_attackable: true,
        ..spawn
    }));
    assert!(!pearl.destination_allowed(V::default(), V(0., 0., 7.999)));
    assert!(pearl.destination_allowed(V::default(), V(0., 0., 8.)));
    assert!(!pearl.throw_allowed(V::default(), None));
    assert!(!pearl.throw_allowed(V::default(), Some(V(0., 0., 8.))));
    assert!(pearl.throw_allowed(V::default(), Some(V(0., 0., 7.999))));
    let target = Rotation::new(45., -20.);
    pearl.enqueue(target);
    pearl.enqueue(Rotation::default());
    assert_eq!(pearl.queue.len(), 1);
    assert_eq!(pearl.begin_tick(Some(-1)), None);
    assert_eq!(pearl.pending.unwrap().waited, 0);
    pearl.enqueue(target);
    assert_eq!(pearl.queue.len(), 1);
    assert_eq!(pearl.begin_tick(Some(2)), None);
    let (e, keep) = pearl.resume(Some(3), 0.5);
    assert!(!keep);
    assert_eq!(
        e,
        Some(Effect::Throw {
            slot: 3,
            rotation: target
        })
    );
    pearl.begin_tick(Some(0));
    for _ in 0..19 {
        assert!(pearl.resume(Some(0), 2.).1);
    }
    assert_eq!(pearl.resume(Some(0), 2.), (None, false));
    pearl.enqueue(target);
    pearl.begin_tick(Some(-1));
    assert_eq!(pearl.resume(None, 0.), (None, false));
    assert!(pearl.pending.is_none());
    pearl.settings.rotate = false;
    pearl.enqueue(target);
    assert_eq!(
        pearl.begin_tick(Some(-1)),
        Some(Effect::Throw {
            slot: -1,
            rotation: target
        })
    );
    let entity = B {
        lo: V(1., -1., -1.),
        hi: V(2., 2., 1.),
    };
    let block = V(0.5, 0., 0.);
    assert_eq!(
        simulate(
            V::default(),
            V(2., 0., 0.),
            -64.,
            &[entity],
            |_, _| Some((false, block)),
            |_| false
        ),
        Some(block)
    );
    assert!(
        simulate(
            V::default(),
            V(2., 0., 0.),
            -64.,
            &[entity],
            |_, end| Some((true, end)),
            |_| false
        )
        .is_some()
    );
    let mut first = None;
    assert!(
        simulate(
            V::default(),
            V(1., 1., 0.),
            -10000.,
            &[],
            |_, end| {
                first.get_or_insert(end);
                Some((true, end))
            },
            |_| false
        )
        .is_none()
    );
    assert_eq!(first, Some(V(0.99, 0.96, 0.)));
    pearl.disabled();
    assert!(pearl.queue.is_empty() && pearl.pending.is_none());
    println!(
        "AutoPearl focused smoke: trigger policies, strict limits, queue/coroutine guards, changed slots, timeout, disabled reset and trajectory physics"
    );
}
