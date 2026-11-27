#![allow(dead_code)]
#[path = "../liquidbounce-behaviors/src/clicking.rs"]
mod clicking;
#[path = "../liquidbounce-behaviors/src/crosshair_clicker.rs"]
mod crosshair_clicker;
use clicking::{Clicker, Cooldown, JavaRandom, Math, Settings, Technique};
use crosshair_clicker::{
    Context, CriticalState, Criticals, Hit, Host, Id, Objective, OnItemUse, State,
};

struct Fixture {
    state: State,
    running: [bool; 3],
    auto: bool,
    now: i64,
    samples: Vec<[i32; 3]>,
    context: Context,
    critical: CriticalState,
    prediction: Result<Option<i32>, String>,
    forecasts: Vec<(i32, f64)>,
    clicks: Vec<Id>,
    weapons: Vec<i32>,
    waits: Vec<(Id, u64)>,
    releases: i32,
}
impl Fixture {
    fn new() -> Self {
        let mut state = State::default();
        for (i, id) in crosshair_clicker::IDS.into_iter().enumerate() {
            *state.clicker(id) = Clicker::new(
                Settings {
                    technique: Technique::Constant,
                    cps: [20, 20],
                    max_per_tick: 2,
                    item_cooldown: None,
                    miss_cooldown: false,
                },
                id.attack(),
                JavaRandom::seeded(i as u64),
                Math::default(),
            );
        }
        state.auto.use_enabled = true;
        state.auto.attacker.criticals = Criticals::Ignore;
        state.trigger.attacker.criticals = Criticals::Ignore;
        Self {
            state,
            running: [true; 3],
            auto: true,
            now: 1000,
            samples: Vec::new(),
            context: Context {
                hit: Hit::Entity(42),
                enemy: true,
                physical_attack: true,
                physical_use: true,
                weapon: 1,
                main_item: "minecraft:stone".into(),
                main_block_item: true,
                ..Context::default()
            },
            critical: CriticalState {
                cooldown: Cooldown {
                    real_ticks: 2.0,
                    reset_ticks: 0.0,
                    server_tick_speed: 1.0,
                    attack_delay: 10.0,
                    has_cooldown: false,
                },
                allows_ignoring_ground: true,
                on_ground: false,
                jumping: false,
                fall_flying: false,
                velocity_y: -0.1,
                fall_distance: 0.0,
                sprinting: false,
                legacy: false,
                jump_height: 0.42,
                blink_in_state: false,
            },
            prediction: Ok(None),
            forecasts: Vec::new(),
            clicks: Vec::new(),
            weapons: Vec::new(),
            waits: Vec::new(),
            releases: 0,
        }
    }
}
impl Host for Fixture {
    fn with_state<T>(&mut self, f: impl FnOnce(&mut State) -> T) -> T {
        f(&mut self.state)
    }
    fn running(&mut self) -> [bool; 3] {
        self.running
    }
    fn auto_running(&mut self) -> bool {
        self.auto
    }
    fn monotonic_millis(&mut self) -> i64 {
        self.samples.push([
            self.state.trigger.clicker.ticks_since_last_click,
            self.state.auto.attack.ticks_since_last_click,
            self.state.auto.use_key.ticks_since_last_click,
        ]);
        self.now
    }
    fn wall_millis(&mut self) -> i64 {
        1000
    }
    fn context(&mut self) -> Option<Context> {
        Some(self.context.clone())
    }
    fn critical_state(&mut self) -> Option<CriticalState> {
        Some(self.critical)
    }
    fn falling_collision(&mut self, ticks: i32, add_y: f64) -> Result<Option<i32>, String> {
        self.forecasts.push((ticks, add_y));
        self.prediction.clone()
    }
    fn auto_weapon_target(&mut self, target: i32) {
        self.weapons.push(target);
    }
    fn release_using(&mut self) {
        self.releases += 1;
        self.context.using = false;
    }
    fn key_click(&mut self, id: Id) {
        self.clicks.push(id);
    }
    fn cooldown_and_miss(&mut self) -> Option<(Cooldown, i32)> {
        Some((self.critical.cooldown, 0))
    }
    fn sample_cooldown(&mut self, range: [f32; 2]) -> f32 {
        range[0]
    }
    fn register_wait(&mut self, id: Id, ticket: u64) {
        self.waits.push((id, ticket));
    }
}

#[test]
fn crosshair_clicker_smoke() {
    // Each source listener samples its own time in Attack, Use, Trigger order.
    let mut h = Fixture::new();
    crosshair_clicker::first_tick(&mut h);
    assert_eq!(h.samples, [[0, 0, 0], [0, 1, 0], [0, 1, 1]]);
    crosshair_clicker::tick(&mut h);
    // Attack and Trigger share one KeyMapping; Use has a separate plan.
    assert_eq!(h.clicks, [Id::Attack, Id::Use]);
    assert_eq!(h.weapons, [42, 42]);
    assert_eq!(h.state.trigger.clicker.click_amount, Some(0));
    assert!(!h.state.attack_key_down(true, [true; 3]));
    assert!(h.state.attack_key_down(true, [false, false, true]));

    let mut player = Fixture::new();
    crosshair_clicker::first_tick(&mut player);
    assert_eq!(
        player
            .state
            .auto
            .attack
            .amount_at(1, player.critical.cooldown),
        1
    );
    crosshair_clicker::player_press(&mut player, true, false, false, true);
    assert_eq!(
        player
            .state
            .auto
            .attack
            .amount_at(1, player.critical.cooldown),
        0
    );
    assert_eq!(
        player
            .state
            .trigger
            .clicker
            .amount_at(1, player.critical.cooldown),
        0
    );
    crosshair_clicker::tick(&mut player);
    assert_eq!(player.clicks, [Id::Attack, Id::Use]);
    let attacker = player.state.auto.attacker;
    player.context.enemy = false;
    assert!(!crosshair_clicker::is_attacking(
        Id::Trigger,
        attacker,
        Objective::Any,
        &player.context
    ));
    assert!(!crosshair_clicker::is_attacking(
        Id::Attack,
        attacker,
        Objective::Enemy,
        &player.context
    ));
    assert!(crosshair_clicker::is_attacking(
        Id::Attack,
        attacker,
        Objective::Entity,
        &player.context
    ));
    player.context.physical_attack = false;
    assert!(!crosshair_clicker::is_attacking(
        Id::Attack,
        attacker,
        Objective::Any,
        &player.context
    ));
    player.context.physical_attack = true;
    player.context.weapon = 2;
    assert!(!crosshair_clicker::is_attacking(
        Id::Attack,
        attacker,
        Objective::Any,
        &player.context
    ));

    let mut wait = Fixture::new();
    wait.running = [true, false, false];
    wait.auto = false;
    wait.context.using = true;
    wait.state.trigger.attacker.delay_post_stop_use = 2;
    crosshair_clicker::first_tick(&mut wait);
    crosshair_clicker::tick(&mut wait);
    let (id, ticket) = wait.waits[0];
    assert!(crosshair_clicker::callback(&mut wait, id, ticket));
    wait.context.using = false;
    assert!(!crosshair_clicker::callback(&mut wait, id, ticket));
    let (id, ticket) = wait.waits[1];
    assert!(crosshair_clicker::callback(&mut wait, id, ticket));
    assert!(!crosshair_clicker::callback(&mut wait, id, ticket));
    assert!(wait.clicks.is_empty());

    let mut stop = Fixture::new();
    stop.running[0] = false;
    stop.context.using = true;
    stop.state.auto.attacker.on_item_use = OnItemUse::Stop;
    stop.state.auto.attacker.delay_post_stop_use = 2;
    crosshair_clicker::first_tick(&mut stop);
    crosshair_clicker::tick(&mut stop);
    assert_eq!(stop.releases, 1);
    assert!(stop.clicks.is_empty()); // Stop ends the entire AutoClicker body.
    let (id, ticket) = stop.waits[0];
    assert!(crosshair_clicker::callback(&mut stop, id, ticket));
    assert!(!crosshair_clicker::callback(&mut stop, id, ticket));
    assert!(stop.clicks.is_empty());

    let mut delay = Fixture::new();
    delay.running = [false, false, true];
    delay.context.physical_attack = false;
    delay.state.auto.delay_start = true;
    crosshair_clicker::first_tick(&mut delay);
    crosshair_clicker::tick(&mut delay);
    let (id, ticket) = delay.waits[0];
    assert_eq!(id, Id::Use);
    assert!(crosshair_clicker::callback(&mut delay, id, ticket));
    assert!(!crosshair_clicker::callback(&mut delay, id, ticket));
    assert!(delay.clicks.is_empty());
    crosshair_clicker::tick(&mut delay);
    assert_eq!(delay.clicks, [Id::Use]);
    delay.clicks.clear();
    delay.context.main_item = "minecraft:compass".into();
    delay.context.main_named = true;
    crosshair_clicker::tick(&mut delay);
    assert!(delay.clicks.is_empty());

    // Smart works independently of Criticals module state and preserves the
    // source's strict landing comparisons and ground jump velocity.
    let mut crit = Fixture::new();
    crit.prediction = Ok(Some(5));
    assert!(crosshair_clicker::critical_ready(
        &mut crit,
        Criticals::Smart
    ));
    assert_eq!(crit.forecasts.pop(), Some((7, 0.0)));
    crit.prediction = Ok(Some(6));
    assert!(!crosshair_clicker::critical_ready(
        &mut crit,
        Criticals::Smart
    ));
    crit.critical.on_ground = true;
    crit.critical.jumping = true;
    crit.prediction = Ok(Some(4));
    assert!(crosshair_clicker::critical_ready(
        &mut crit,
        Criticals::Smart
    ));
    let (ticks, add_y) = crit.forecasts.pop().unwrap();
    assert_eq!(ticks, 11); // max(nextCrit=6.5, untilFall=5.25) + 5.
    assert_eq!(add_y, f64::from(0.42f32));
    crit.prediction = Err("missing local simulation".into());
    assert!(!crosshair_clicker::critical_ready(
        &mut crit,
        Criticals::Smart
    ));
    crit.critical.on_ground = false;
    crit.critical.cooldown.real_ticks = 9.0;
    crit.critical.fall_distance = 1.0;
    assert!(crosshair_clicker::critical_ready(
        &mut crit,
        Criticals::Always
    ));
    crit.critical.sprinting = true;
    assert!(!crosshair_clicker::critical_ready(
        &mut crit,
        Criticals::Always
    ));
    crit.critical.legacy = true;
    assert!(crosshair_clicker::critical_ready(
        &mut crit,
        Criticals::Always
    ));

    // Imported BlocksForIgnore remains authoritative when native defaults
    // become available after construction.
    let mut imported = State::default();
    imported.blocks_configured = true;
    imported
        .auto
        .blocks_for_ignore
        .insert("minecraft:stone".into());
    imported.initialize(Math::default(), ["minecraft:oak_door".into()].into());
    assert_eq!(
        imported.auto.blocks_for_ignore,
        ["minecraft:stone".into()].into()
    );
}
