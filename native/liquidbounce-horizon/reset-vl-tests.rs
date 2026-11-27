use crate::{
    coroutine_ticker::{CoroutineTicker, WaitOwner},
    reset_vl::{Effect, ResetVl, TIMER_PRIORITY},
    timer::Timer,
};
use serde_json::{Value, json};

struct Fixture {
    module: ResetVl,
    ticker: CoroutineTicker,
    timer: Timer,
    callbacks: Vec<Value>,
    enabled: bool,
    in_game: bool,
    same_thread: bool,
    other: bool,
    ground: bool,
    velocity: [f64; 3],
    events: Vec<Value>,
}
impl Default for Fixture {
    fn default() -> Self {
        Self {
            module: ResetVl::default(),
            ticker: CoroutineTicker::default(),
            timer: Timer::default(),
            callbacks: vec![],
            enabled: false,
            in_game: true,
            same_thread: true,
            other: false,
            ground: false,
            velocity: [7.0, -0.0, -8.0],
            events: vec![],
        }
    }
}
impl Fixture {
    fn running(&self) -> bool {
        self.enabled && self.in_game
    }
    fn event(&mut self, kind: &str, value: String) {
        self.events
            .push(json!({"kind":kind,"value":value,"count":self.module.count}));
    }
    fn enabled(&mut self, value: bool) {
        if self.enabled == value {
            return;
        }
        self.enabled = value;
        self.event("enabled", format!("ResetVL:{value}"));
        if !value {
            self.module.cancel_sequences();
        }
    }
    fn effects(&mut self, mut effect: Option<Effect>) {
        while let Some(current) = effect {
            match current {
                Effect::RequestTimer(speed) => {
                    self.timer.request_speed(speed, TIMER_PRIORITY, 1, 1)
                }
                Effect::Velocity(y) => {
                    self.velocity = [0.0, y, 0.0];
                    self.event(
                        "velocity",
                        self.velocity.map(|v| v.to_bits().to_string()).join(","),
                    );
                }
                Effect::Disable => self.enabled(false),
                Effect::NotifySuccess => self.event(
                    "notification",
                    "ResetVL|Successfully reset violations|SUCCESS".into(),
                ),
            }
            effect = self.module.after_effect(current, self.ground);
            for id in self.module.take_wait_registrations() {
                self.ticker.register(WaitOwner::ResetVl, id);
            }
        }
    }
    fn emit(&mut self) {
        self.timer.game_tick();
        let effect = self.module.start_tick_handler(self.running());
        self.effects(effect);
    }
    fn tick(&mut self, ground: bool, nested: bool) {
        self.ground = ground;
        let ticker = self.ticker.clone();
        let mut batch = ticker.begin_tick().unwrap();
        while let Some(task) = batch.next_task() {
            match task.owner {
                WaitOwner::ResetVl => {
                    let (keep, effect) =
                        self.module.advance_callback(task.callback, self.running());
                    self.effects(effect);
                    batch.finish_current(keep);
                }
                WaitOwner::InventoryManager => {
                    self.mutation(&self.callbacks[task.callback as usize].clone());
                    batch.finish_current(false);
                }
                _ => panic!("unexpected oracle owner"),
            }
        }
        // Keep the outer batch guard through nested Minecraft.tick + GameTick.
        if nested {
            assert!(ticker.begin_tick().is_none());
            self.emit();
        }
        self.emit();
    }
    fn mutation(&mut self, action: &Value) {
        match action["op"].as_str().unwrap() {
            "enabled" => self.enabled(action["value"].as_bool().unwrap()),
            "guard" => self.in_game = action["value"].as_bool().unwrap(),
            "times" => self.module.settings.times = action["value"].as_i64().unwrap() as i32,
            "timer" => {
                self.module.settings.timer =
                    f32::from_bits(action["bits"].as_str().unwrap().parse().unwrap())
            }
            "count" => self.module.count = action["value"].as_i64().unwrap() as i32,
            "ground" => self.ground = action["value"].as_bool().unwrap(),
            "sameThread" => self.same_thread = action["value"].as_bool().unwrap(),
            "world" => self.timer.world_change(),
            "velocity" => {
                self.velocity = std::array::from_fn(|i| {
                    f64::from_bits(action["bits"][i].as_str().unwrap().parse().unwrap())
                })
            }
            "other" => {
                let enabled = action["enabled"].as_bool().unwrap();
                if self.other != enabled {
                    self.other = enabled;
                    self.event("enabled", format!("Other:{enabled}"));
                }
                if let Some(bits) = action["bits"].as_str() {
                    self.timer.request_speed(
                        f32::from_bits(bits.parse().unwrap()),
                        action["priority"].as_i64().unwrap() as i32,
                        2,
                        action["reset"].as_i64().unwrap() as i32,
                    );
                }
            }
            _ => panic!("unknown mutation {action}"),
        }
    }
    fn step(&mut self, action: &Value) -> Value {
        match action["op"].as_str().unwrap() {
            "tick" => self.tick(
                action["ground"].as_bool().unwrap(),
                action["nested"].as_bool().unwrap(),
            ),
            "emit" => self.emit(),
            "callback" => {
                let id = self.callbacks.len() as u64;
                self.callbacks.push(action["action"].clone());
                self.ticker.register(WaitOwner::InventoryManager, id);
            }
            _ => self.mutation(action),
        }
        let running = self.running();
        let other = self.other && self.in_game;
        let speed = self.timer.speed(self.same_thread, &mut |id| {
            if id == 1 { running } else { other }
        });
        json!({"count":self.module.count,"enabled":self.enabled,"running":running,
            "busy":self.module.busy(),"speed":speed.to_bits().to_string(),
            "velocity":self.velocity.map(|v|v.to_bits().to_string()),
            "events":std::mem::take(&mut self.events),"timerTick":self.timer.requests.current_tick()})
    }
}

#[test]
fn every_step_matches_unchanged_kotlin_coroutines_and_timer() {
    let inputs: Vec<Vec<Value>> = serde_json::from_str(include_str!("inputs.json")).unwrap();
    let oracle: Vec<Vec<Value>> = serde_json::from_str(include_str!("oracle.json")).unwrap();
    assert_eq!(inputs.len(), oracle.len());
    for (case, (actions, expected)) in inputs.iter().zip(&oracle).enumerate() {
        let mut fixture = Fixture::default();
        for (step, (action, expected)) in actions.iter().zip(expected).enumerate() {
            assert_eq!(
                &fixture.step(action),
                expected,
                "case {case} step {step} action {action}"
            );
        }
    }
}

#[test]
fn synchronous_disable_cancellation_keeps_notification_then_clears_count() {
    let mut fixture = Fixture::default();
    fixture.module.settings.times = 1;
    fixture.enabled(true);
    fixture.tick(true, false);
    fixture.tick(false, false);
    assert!(!fixture.enabled);
    assert_eq!(fixture.module.count, 0);
    assert_eq!(fixture.events[fixture.events.len() - 2]["count"], 1);
    assert_eq!(fixture.events.last().unwrap()["kind"], "notification");
    assert_eq!(fixture.events.last().unwrap()["count"], 1);
}

#[test]
fn disabled_waiter_does_not_resume_after_reenable_or_reset_count() {
    let mut fixture = Fixture::default();
    fixture.module.count = 4;
    fixture.enabled(true);
    fixture.tick(true, false);
    fixture.enabled(false);
    fixture.enabled(true);
    fixture.tick(false, false);
    assert_eq!(fixture.module.count, 4);
    assert_eq!(
        fixture.velocity.map(f64::to_bits),
        [0.0_f64.to_bits(), 0.1_f64.to_bits(), 0.0_f64.to_bits()]
    );
}
