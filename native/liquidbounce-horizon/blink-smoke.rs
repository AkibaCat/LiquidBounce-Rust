#[path = "../liquidbounce-behaviors/src/blink.rs"]
mod blink;
use blink::*;
#[test]
fn threshold_direction_and_shared_queue_operations() {
    let mut b = Blink::default();
    assert!(b.queues(true, true));
    assert!(!b.queues(true, false));
    b.settings.auto_reset = true;
    b.settings.reset_after = 1;
    assert_eq!(b.movement(), None);
    assert_eq!(b.movement(), Some(0));
    assert_eq!(b.ticks, 0);
    let mut q = Queue::default();
    for (n, movement, position, outgoing) in [
        (0, false, None, true),
        (1, true, Some([1., 2., 3.]), true),
        (2, true, None, true),
        (3, false, None, false),
        (4, true, Some([4., 5., 6.]), true),
    ] {
        q.0.push_back(Snapshot {
            packet: n,
            outgoing,
            millis: 10,
            position,
            movement,
        });
    }
    assert_eq!(q.positions().len(), 2);
    assert!(q.is_above_time(15, 5));
    assert_eq!(
        q.flush_count(1)
            .iter()
            .map(|s| s.packet)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    let (at, p) = q.cancel();
    assert_eq!(at, Some([4., 5., 6.]));
    assert_eq!(p[0].packet, 3);
    assert!(q.0.is_empty());
}
