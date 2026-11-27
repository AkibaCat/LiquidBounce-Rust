use crate::{inventory_manager::*, random::ThreadRandom, vomit::Constraints};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Clone)]
struct Program {
    owner: u64,
    priority: i32,
    chains: Vec<Vec<Action>>,
    index: usize,
}
struct Fixture {
    manager: Manager,
    c: Context,
    owners: BTreeMap<u64, OwnerState>,
    programs: BTreeMap<u64, Program>,
    rng: ThreadRandom,
    effects: Vec<String>,
    waiters: Vec<u64>,
    screen_serial: u64,
    inventory_contents: Vec<Stack>,
    inventory_carried: Stack,
    nearest: Option<Vec<String>>,
    mouse_pixels: [f64; 2],
    move_cursor: bool,
    mouse_grabbed: bool,
    screen_hidden: bool,
    screen_left: i32,
    screen_top: i32,
    screen_width: i32,
    gui_width: i32,
}
fn int(v: &Value, key: &str, default: i32) -> i32 {
    v[key].as_i64().map_or(default, |v| v as i32)
}
fn boolean(v: &Value, key: &str, default: bool) -> bool {
    v[key].as_bool().unwrap_or(default)
}
fn slot(v: &Value) -> Slot {
    let s = v.as_str().unwrap();
    if let Some(index) = s.strip_prefix('p') {
        Slot::Player(index.parse().unwrap())
    } else {
        Slot::Container(s[1..].parse().unwrap())
    }
}
fn stack(v: &Value) -> Stack {
    Stack {
        components: v["item"].as_str().unwrap_or("stone").into(),
        count: int(v, "count", 64),
        wire: String::new(),
    }
}
fn string(s: &Stack) -> String {
    if s.empty() {
        String::new()
    } else {
        format!("{}:{}", s.components, s.count)
    }
}
fn input(i: i32) -> Input {
    match i {
        0 => Input::Pickup,
        1 => Input::QuickMove,
        2 => Input::Swap,
        3 => Input::Clone,
        4 => Input::Throw,
        5 => Input::QuickCraft,
        6 => Input::PickupAll,
        _ => panic!(),
    }
}
impl Fixture {
    fn new(thread: u64) -> Self {
        let inventory_menu = Geometry {
            object: 1,
            slots: (0..46)
                .map(|i| Position {
                    x: (i % 9) as f32 * 18.,
                    y: (i / 9) as f32 * 18.,
                })
                .collect(),
        };
        Self {
            manager: Manager::default(),
            c: Context {
                in_game: true,
                creative: true,
                no_screen: true,
                menu: MenuState {
                    container_id: 0,
                    stacks: vec![Stack::default(); 46],
                    carried: Stack::default(),
                },
                inventory_menu,
                player_stacks: vec![Stack::default(); 41],
                requirements: Requirements {
                    rotation_matches: true,
                    breaking: Some(false),
                    ..Default::default()
                },
                ..Default::default()
            },
            owners: BTreeMap::new(),
            programs: BTreeMap::new(),
            rng: ThreadRandom::seeded(305419896, thread),
            effects: vec![],
            waiters: vec![],
            screen_serial: 1,
            inventory_contents: vec![Stack::default(); 46],
            inventory_carried: Stack::default(),
            nearest: None,
            mouse_pixels: [0.0; 2],
            move_cursor: false,
            mouse_grabbed: false,
            screen_hidden: false,
            screen_left: 0,
            screen_top: 0,
            screen_width: 854,
            gui_width: 427,
        }
    }
    fn owner(&mut self, id: u64) -> &mut OwnerState {
        self.owners.entry(id).or_insert(OwnerState {
            running: true,
            guards_pass: true,
            constraints: Constraints::default(),
        })
    }
    fn refresh_players(&mut self) {
        for index in 0..41u8 {
            let slot = Slot::Player(index);
            if let Some(id) = slot.server_id(self.c.screen.as_ref().filter(|s| s.container_id != 0))
                && let Some(value) = self.c.menu.stacks.get(id as usize)
            {
                self.c.player_stacks[index as usize] = value.clone();
            }
        }
    }
    fn set_player(&mut self, index: u8, value: Stack) {
        self.c.player_stacks[index as usize] = value.clone();
        if let Some(id) =
            Slot::Player(index).server_id(self.c.screen.as_ref().filter(|s| s.container_id != 0))
            && let Some(s) = self.c.menu.stacks.get_mut(id as usize)
        {
            *s = value;
        }
    }
    fn action(&self, v: &Value) -> Action {
        match v["kind"].as_str().unwrap() {
            "click" => Action::Click {
                screen: if boolean(v, "screen", false) {
                    self.c.screen.clone()
                } else {
                    None
                },
                slot: slot(&v["slot"]),
                button: int(v, "button", 1),
                input: input(int(v, "input", 4)),
            },
            "creative" => Action::Creative {
                stack: CreativeStack::Existing(stack(v)),
                slot: v.get("slot").map(slot),
            },
            "use" => Action::UseItem {
                slot: int(v, "slot", 0) as u8,
            },
            "close" => Action::CloseScreen {
                container_id: self.c.menu.container_id,
            },
            _ => panic!("action {v}"),
        }
    }
    fn submit(&mut self, id: u64) {
        let p = self.programs[&id].clone();
        self.manager.submit(Request::new(
            id,
            p.owner,
            p.priority,
            p.chains[p.index].clone(),
            &self.c,
            &self.manager,
        ));
    }
    fn outcome(&mut self, outcome: &Outcome) {
        match outcome {
            Outcome::Performed { .. } => {}
            Outcome::Cancelled { request } => {
                if self.programs.remove(request).is_some() {
                    self.effects.push(format!("cancel:{request}"));
                    self.manager.end_plan(*request);
                }
            }
            Outcome::Completed { request } => {
                let p = self.programs[request].clone();
                let owner = self.owners[&p.owner].clone();
                if !owner.running || !owner.guards_pass {
                    self.programs.remove(request);
                    self.effects.push(format!("cancel:{request}"));
                    self.manager.end_plan(*request);
                    return;
                }
                self.effects
                    .push(format!("completed:{request}:{}", p.index));
                if p.index + 1 == p.chains.len() {
                    self.effects.push(format!("done:{request}"));
                    self.programs.remove(request);
                    self.manager.end_plan(*request);
                } else {
                    self.programs.get_mut(request).unwrap().index += 1;
                    self.submit(*request);
                }
            }
        }
    }
    fn native(&mut self, effect: &Effect) -> bool {
        match effect {
            Effect::Outcome(o) => self.outcome(o),
            Effect::OpenInventory => self.effects.push("open".into()),
            Effect::CloseInventory => {
                self.effects.push("close".into());
                if !self.manager.outgoing_close(0) {
                    self.effects.push("cancel-packet".into());
                }
            }
            Effect::MoveCursor(p) => {
                let scale = self.screen_width as f32 / self.gui_width as f32;
                self.mouse_pixels = [
                    ((self.screen_left as f32 + p.x) * scale) as f64,
                    ((self.screen_top as f32 + p.y) * scale) as f64,
                ];
                self.effects.push(format!(
                    "cursor:{}:{}",
                    self.mouse_pixels[0].to_bits() as i64,
                    self.mouse_pixels[1].to_bits() as i64
                ));
            }
            Effect::Action(action) => match action {
                Action::Creative {
                    stack: CreativeStack::Existing(stack),
                    slot,
                } => {
                    if self.c.creative {
                        let id = slot.map_or(-1, |s| s.server_id(None).unwrap());
                        if slot.is_some()
                            || (!stack.empty()
                                && !self.c.inventory_screen
                                && !self.c.container_screen)
                        {
                            self.effects.push(format!(
                                "creative:{id}:{}:{}",
                                stack.components, stack.count
                            ));
                        }
                    }
                }
                Action::Click {
                    screen,
                    slot,
                    button,
                    input,
                } => {
                    let id = screen.as_ref().map_or(0, |s| s.container_id);
                    let server = slot.server_id(screen.as_ref()).unwrap();
                    let ordinal = match input {
                        Input::Pickup => 0,
                        Input::QuickMove => 1,
                        Input::Swap => 2,
                        Input::Clone => 3,
                        Input::Throw => 4,
                        Input::QuickCraft => 5,
                        Input::PickupAll => 6,
                    };
                    self.effects
                        .push(format!("click:{id}:{server}:{button}:{ordinal}"));
                    if id == self.c.menu.container_id {
                        if let Some(old) = self.c.menu.stacks.get(server as usize).cloned() {
                            match input {
                                Input::Throw if self.c.menu.carried.empty() => {
                                    self.c.menu.stacks[server as usize] =
                                        if *button == 0 && old.count > 1 {
                                            Stack {
                                                count: old.count - 1,
                                                ..old
                                            }
                                        } else {
                                            Stack::default()
                                        };
                                }
                                Input::QuickMove => {
                                    self.c.menu.stacks[server as usize] = Stack::default()
                                }
                                Input::Pickup => {
                                    self.c.menu.stacks[server as usize] =
                                        std::mem::replace(&mut self.c.menu.carried, old);
                                }
                                Input::Swap => {
                                    let other = self.c.player_stacks[*button as usize].clone();
                                    self.c.menu.stacks[server as usize] = other;
                                    self.set_player(*button as u8, old);
                                }
                                _ => {}
                            }
                            self.refresh_players();
                        }
                        self.manager.sent_click(id);
                    }
                }
                Action::CloseScreen { container_id } => {
                    if !self.manager.outgoing_close(*container_id) {
                        self.effects.push("cancel-packet".into());
                    }
                    self.screen("none", 0, 0);
                }
                Action::UseItem { slot } => {
                    self.effects.push(format!("select:{slot}:1"));
                    self.effects.push(format!("use:{}", i32::from(*slot == 40)));
                }
                _ => panic!(),
            },
        }
        true
    }
    fn drive(&mut self) {
        loop {
            let scale = self.screen_width as f32 / self.gui_width as f32;
            self.c.move_visible_cursor =
                self.move_cursor && !self.mouse_grabbed && !self.screen_hidden;
            self.c.visible_cursor = Position {
                x: self.mouse_pixels[0] as f32 / scale - self.screen_left as f32,
                y: self.mouse_pixels[1] as f32 / scale - self.screen_top as f32,
            };
            let owners = &self.owners;
            let emission = self
                .manager
                .step(&self.c, &mut |id| owners[&id].clone(), &mut self.rng)
                .unwrap();
            self.waiters.extend(self.manager.take_wait_registrations());
            let Some(emission) = emission else {
                return;
            };
            let success = self.native(&emission.effect);
            assert!(self.manager.acknowledge(emission.ticket, success, &self.c));
        }
    }
    fn screen(&mut self, kind: &str, id: i32, own: usize) {
        if self.c.menu.container_id == 0 {
            self.inventory_contents = self.c.menu.stacks.clone();
            self.inventory_carried = self.c.menu.carried.clone();
        }
        self.screen_serial += 1;
        self.screen_left = 0;
        self.screen_top = 0;
        self.c.inventory_screen = kind == "inventory";
        self.c.container_screen = kind == "container";
        self.c.no_screen = kind == "none";
        let stacks = if kind == "container" {
            let mut s = vec![Stack::default(); own];
            s.extend((9..36).chain(0..9).map(|i| self.c.player_stacks[i].clone()));
            s
        } else {
            (0..46)
                .map(|i| {
                    let player = match i {
                        9..=35 => Some(i),
                        36..=44 => Some(i - 36),
                        5..=8 => Some(44 - i),
                        45 => Some(40),
                        _ => None,
                    };
                    player.map_or_else(
                        || self.inventory_contents[i].clone(),
                        |p| self.c.player_stacks[p].clone(),
                    )
                })
                .collect()
        };
        self.c.menu = MenuState {
            container_id: if kind == "container" { id } else { 0 },
            stacks,
            carried: if kind == "container" {
                Stack::default()
            } else {
                self.inventory_carried.clone()
            },
        };
        self.c.screen = if kind == "none" || kind == "other" {
            None
        } else {
            Some(Screen {
                object: self.screen_serial,
                container_id: self.c.menu.container_id,
                geometry: if kind == "container" {
                    Geometry {
                        object: self.screen_serial,
                        slots: (0..own + 36)
                            .map(|i| Position {
                                x: (i % 9) as f32 * 18.,
                                y: (i / 9) as f32 * 18.,
                            })
                            .collect(),
                    }
                } else {
                    self.c.inventory_menu.clone()
                },
                non_player_slots: if kind == "container" {
                    (0..own as i32).collect()
                } else {
                    vec![]
                },
            })
        };
        if self.c.screen.is_some() {
            self.manager.opened();
        }
    }
    fn command(&mut self, v: &Value) {
        match v["cmd"].as_str().unwrap() {
            "tick" => {
                let waiters = std::mem::take(&mut self.waiters);
                for id in waiters {
                    self.manager.advance_callback(id);
                    self.drive();
                }
                self.manager.begin_tick(&self.c);
                self.drive();
            }
            "advance" => {
                let end = self.c.now + i64::from(int(v, "ms", 0));
                while let Some(next) = self.manager.next_wakeup(self.c.now).filter(|n| *n <= end) {
                    self.c.now = next;
                    self.drive();
                }
                self.c.now = end;
            }
            "owner" => {
                let o = self.owner(int(v, "id", 0) as u64);
                o.running = boolean(v, "running", true);
                o.guards_pass = boolean(v, "guard", true);
            }
            "settings" => {
                self.manager.response_time = int(v, "responseTime", self.manager.response_time);
                self.manager.lock_after_rejections =
                    int(v, "lockAfterRejections", self.manager.lock_after_rejections);
            }
            "cursor" => {
                self.manager.cursor =
                    v.get("value")
                        .filter(|value| !value.is_null())
                        .map(|value| Position {
                            x: value[0].as_f64().unwrap() as f32,
                            y: value[1].as_f64().unwrap() as f32,
                        });
            }
            "nearestFirst" => {
                let items = v["slots"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|value| value.as_str().unwrap().to_owned())
                    .collect::<Vec<_>>();
                self.nearest = Some(
                    self.manager
                        .nearest_first(&items, &self.c.inventory_menu, |value| {
                            slot(&Value::String(value.clone()))
                        })
                        .unwrap(),
                );
            }
            "visiblecursor" => {
                self.move_cursor = boolean(v, "enabled", self.move_cursor);
                self.mouse_grabbed = boolean(v, "grabbed", self.mouse_grabbed);
                self.screen_hidden = boolean(v, "hidden", self.screen_hidden);
                self.mouse_pixels[0] = v["x"].as_f64().unwrap_or(self.mouse_pixels[0]);
                self.mouse_pixels[1] = v["y"].as_f64().unwrap_or(self.mouse_pixels[1]);
                self.screen_width = int(v, "width", self.screen_width);
                self.gui_width = int(v, "guiWidth", self.gui_width);
                if self.c.screen.is_some() {
                    self.screen_left = int(v, "left", self.screen_left);
                    self.screen_top = int(v, "top", self.screen_top);
                }
            }
            "constraints" => {
                let c = &mut self.owner(int(v, "owner", 1) as u64).constraints;
                for name in ["start", "click", "close", "miss"] {
                    if let Some(r) = v.get(name) {
                        let r = [r[0].as_i64().unwrap() as i32, r[1].as_i64().unwrap() as i32];
                        match name {
                            "start" => c.start_delay = r,
                            "click" => c.click_delay = r,
                            "close" => c.close_delay = r,
                            _ => c.miss_chance = r,
                        }
                    }
                }
                if let Some(m) = v.get("multiplier") {
                    c.slot_multiplier = m.as_f64().unwrap() as f32;
                }
                if let Some(m) = v.get("requires") {
                    c.requirements = m.as_u64().unwrap() as u8;
                }
            }
            "set" => {
                let slot = slot(&v["slot"]);
                let value = stack(v);
                if let Slot::Player(index) = slot {
                    self.set_player(index, value);
                } else {
                    let id = slot.server_id(self.c.screen.as_ref()).unwrap();
                    self.c.menu.stacks[id as usize] = value;
                    self.refresh_players();
                }
            }
            "carried" => self.c.menu.carried = stack(v),
            "screen" => self.screen(
                v["kind"].as_str().unwrap(),
                int(v, "id", 7),
                int(v, "slots", 27) as usize,
            ),
            "plan" => {
                let id = int(v, "id", 0) as u64;
                let owner = int(v, "owner", 1) as u64;
                self.owner(owner);
                let chains = v["chains"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|c| {
                        c.as_array()
                            .unwrap()
                            .iter()
                            .map(|a| self.action(a))
                            .collect()
                    })
                    .collect();
                self.programs.insert(
                    id,
                    Program {
                        owner,
                        priority: int(v, "priority", 1),
                        chains,
                        index: 0,
                    },
                );
                self.submit(id);
            }
            "state" => {
                self.c.creative = boolean(v, "creative", self.c.creative);
                self.c.in_game = boolean(v, "game", self.c.in_game);
                if let Some(m) = v.get("moving") {
                    self.c.requirements.movement = [if m.as_bool().unwrap() { 1. } else { 0. }, 0.];
                }
                self.c.requirements.jumping = boolean(v, "jumping", self.c.requirements.jumping);
                self.c.requirements.rotation_matches =
                    boolean(v, "rotation", self.c.requirements.rotation_matches);
                self.c.requirements.using_item =
                    boolean(v, "using", self.c.requirements.using_item);
                self.c.requirements.breaking = Some(boolean(
                    v,
                    "breaking",
                    self.c.requirements.breaking.unwrap(),
                ));
                self.c.requirements.combat = boolean(v, "combat", self.c.requirements.combat);
                self.c.server_controlled_inventory =
                    boolean(v, "serverControlled", self.c.server_controlled_inventory);
            }
            "packet" => match v["kind"].as_str().unwrap() {
                "open-pending" => self.manager.may_open_container(self.c.now),
                "opened" => self.manager.container_opened(),
                "out-close" => {
                    if !self.manager.outgoing_close(int(v, "id", 0)) {
                        self.effects.push("cancel-packet".into());
                    }
                }
                "in-close" => self.manager.incoming_close(),
                "slot" => {
                    let id = int(v, "id", 0);
                    let slot = int(v, "slot", 0);
                    let stack = stack(v);
                    if id == 0 {
                        self.manager.resync_slot(id, slot, &stack, self.c.now);
                    }
                    if id == self.c.menu.container_id {
                        self.c.menu.stacks[slot as usize] = stack;
                        self.refresh_players();
                    }
                }
                "content" => {
                    self.manager
                        .resync_content(int(v, "id", 0), &self.c.menu.stacks, self.c.now)
                }
                _ => panic!(),
            },
            "world" => {
                self.manager.world_change();
                // The original handler synchronously resumes cancelled plan
                // continuations. step drains outcomes before checking phase;
                // drive does not call begin_tick or start an idle executor.
                self.drive();
            }
            _ => panic!("command {v}"),
        }
        // Packet callbacks may run the queued cancellation, but an idle executor
        // does not start a new inventory loop until the next GameTick.
        self.manager.poll_timers(self.c.now);
    }
    fn row(&mut self) -> Value {
        let debug = format!("{:?}", self.rng);
        let seed = debug
            .split("seed: ")
            .nth(1)
            .unwrap()
            .split(',')
            .next()
            .unwrap();
        json!({"now":self.c.now,"seed":seed,"open":self.manager.inventory_open_server_side,"last":self.manager.last_clicked_slot,
            "locked":self.manager.locked_count(),"revert":self.manager.revert_pending(self.c.now),"containerPending":self.manager.container_open_pending(self.c.now),
            "usePending":self.manager.use_item_pending(self.c.now),"sourceMillis":self.c.now,"nearest":self.nearest,
            "mouse":self.mouse_pixels.map(|v|v.to_bits() as i64),
            "pending":self.manager.pending(),"effects":std::mem::take(&mut self.effects),"cursor":self.manager.cursor.map(|p|[p.x.to_bits() as i32,p.y.to_bits() as i32]),
            "stacks":self.c.menu.stacks.iter().map(string).collect::<Vec<_>>(),"carried":string(&self.c.menu.carried)})
    }
}
#[test]
fn production_manager_matches_unchanged_kotlin_coroutine_actions_and_random_state() {
    let source: Value = serde_json::from_str(include_str!("source-vectors.json")).unwrap();
    let commands: Value = serde_json::from_str(include_str!("commands.json")).unwrap();
    let mut f = Fixture::new(source["thread"].as_u64().unwrap());
    for (index, (command, expected)) in commands
        .as_array()
        .unwrap()
        .iter()
        .zip(source["rows"].as_array().unwrap())
        .enumerate()
    {
        f.command(command);
        let actual = f.row();
        for key in [
            "now",
            "seed",
            "open",
            "last",
            "locked",
            "revert",
            "pending",
            "effects",
            "cursor",
            "stacks",
            "carried",
            "containerPending",
            "usePending",
            "sourceMillis",
            "nearest",
            "mouse",
        ] {
            if expected.get(key).is_none() {
                continue;
            }
            assert_eq!(
                actual[key], expected[key],
                "row {index} key {key} command {command}"
            );
        }
    }
}
