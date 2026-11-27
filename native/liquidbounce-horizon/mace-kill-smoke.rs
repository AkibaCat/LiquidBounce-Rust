#[path = "../liquidbounce-behaviors/src/mace_kill.rs"]
mod mace_kill;
use mace_kill::{Host, MaceKill};

struct World {
    mace: bool,
    clear_height: i32,
    position: [f64; 3],
    queries: Vec<i32>,
    packets: Vec<(Option<[f64; 3]>, bool)>,
    ground_reads: usize,
}
impl Default for World {
    fn default() -> Self {
        Self {
            mace: true,
            clear_height: 22,
            position: [1.0, 80.0, 3.0],
            queries: vec![],
            packets: vec![],
            ground_reads: 0,
        }
    }
}
impl Host for World {
    fn holding_mace(&mut self) -> bool {
        self.mace
    }
    fn determine_height(&mut self, maximum: i32) -> i32 {
        mace_kill::height::highest_clear::<()>(maximum, |height| {
            self.queries.push(height);
            Ok(height == self.clear_height)
        })
        .unwrap()
    }
    fn on_ground(&mut self) -> bool {
        self.ground_reads += 1;
        self.ground_reads % 2 == 1
    }
    fn position(&mut self) -> [f64; 3] {
        self.position
    }
    fn warp(&mut self, position: Option<[f64; 3]>, ground: bool) {
        self.packets.push((position, ground));
        self.position[0] += 1.0;
    }
}
#[test]
fn mace_attacks_search_highest_landing_and_preserve_live_warp_order() {
    let mut high = World::default();
    MaceKill::default().attack(true, &mut high);
    assert_eq!(high.queries, [22]);
    assert_eq!(high.ground_reads, 0);
    assert_eq!(
        high.packets,
        [
            (None, false),
            (None, false),
            (None, false),
            (Some([4.0, 102.0, 3.0]), false),
            (Some([5.0, 80.0, 3.0]), false)
        ]
    );

    let mut low = World {
        clear_height: 10,
        ..World::default()
    };
    MaceKill { fall_height: 12 }.attack(true, &mut low);
    assert_eq!(low.queries, [12, 11, 10]);
    assert_eq!(
        low.packets,
        [
            (None, true),
            (None, false),
            (Some([3.0, 90.0, 3.0]), false),
            (Some([4.0, 80.0, 3.0]), false)
        ]
    );

    let mut blocked = World {
        clear_height: 0,
        ..World::default()
    };
    MaceKill { fall_height: 2 }.attack(true, &mut blocked);
    assert_eq!(blocked.queries, [2, 1]);
    assert_eq!(blocked.packets.len(), 4);
    assert_eq!(blocked.packets[2], (Some([3.0, 80.0, 3.0]), false));

    let mut idle = World {
        mace: false,
        ..World::default()
    };
    MaceKill::default().attack(true, &mut idle);
    idle.mace = true;
    MaceKill::default().attack(false, &mut idle);
    assert!(idle.queries.is_empty() && idle.packets.is_empty());
    assert_eq!(
        mace_kill::height::highest_clear::<()>(0, |_| panic!("empty range queried")),
        Ok(0)
    );
    assert_eq!(
        mace_kill::height::highest_clear::<&str>(22, |_| Err("bad shape")),
        Err("bad shape")
    );
}
