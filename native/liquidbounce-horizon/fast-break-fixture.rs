#![cfg(test)]
#![allow(dead_code)]
extern crate self as horizon_client;
extern crate self as liquidbounce_behaviors;
mod config;
mod fast_break;
mod fast_break_bridge;
mod fast_break_packets;
mod no_swing;
mod packet_bridge;
pub mod packets;
use horizon_protocol::{
    connection::{ConnWriter, Connection, Pipe},
    packets::{Empty, play},
};
use play::Serverbound as S;
use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
    marker::PhantomData,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
pub mod mod_api {
    pub use horizon_protocol::packets::play::{
        Clientbound as ClientboundPlayPacket, Serverbound as ServerboundPlayPacket,
    };
    pub trait RenderHooks: Send + Sync {}
    #[allow(unused_imports)]
    mod generated_host {
        include!(concat!(env!("OUT_DIR"), "/mod_api_generated_host.rs"));
    }
    pub use generated_host::*;
}
struct Session {
    writer: Arc<Mutex<Option<ConnWriter>>>,
    configuring: Arc<AtomicBool>,
    serial: u64,
    debug: bool,
}
impl Session {
    /* SESSION_METHODS */
    fn serial(&self) -> u64 {
        self.serial
    }
}
struct Player {
    server_copy: bool,
    replaying: bool,
}
impl Player {
    /* IDENTITY */
}
#[derive(Clone)]
struct ItemStack {
    item: Box<str>,
    count: u32,
}
#[derive(Default)]
struct Items {
    item_tags: HashMap<String, HashSet<String>>,
}
impl Items {
    fn can_destroy_block(&self, _: Option<&ItemStack>, _: bool) -> bool {
        true
    }
}
struct Facts;
impl Facts {
    fn is_air(&self, _: u32) -> bool {
        false
    }
}
#[derive(Default)]
struct Resources {
    items: Items,
}
impl Resources {
    fn items(&self) -> &Items {
        &self.items
    }
    fn facts(&self) -> Facts {
        Facts
    }
}
struct World;
impl World {
    fn block(&self, _: i32, _: i32, _: i32) -> u32 {
        1
    }
}
#[derive(Clone, Copy)]
struct Hit {
    pos: [i32; 3],
    face: [i32; 3],
}
#[derive(Clone)]
struct Mining {
    pos: [i32; 3],
    progress: f32,
    per_tick: f32,
    ticks: u32,
}
#[derive(Default)]
struct Clicks {
    destroying: LocalDestroying,
}
#[derive(Default)]
struct LocalDestroying {
    delay: i32,
    progress: f32,
}
mod remote {
    use super::*;
    pub(super) struct Remote {
        pub session: Session,
        pub(super) joined: bool,
        pub(super) reconfiguring: Option<()>,
        /* REMOTE_FIELDS */
        destroying_item: String,
        is_destroying: bool,
    }
    pub(super) fn remote(session: Session) -> Remote {
        Remote {
            session,
            joined: true,
            reconfiguring: None,
            destroy_delay: 4,
            destroying: Some(([3, 42, -5], 1, 0.85)),
            destroying_item: "minecraft:diamond_pickaxe".into(),
            is_destroying: true,
        }
    }
    pub(super) fn mine(game: &mut Game<'_>, start: bool) {
        game.remote_mining(start);
    }
    impl Game<'_> {
        fn stop_remote_mining(&mut self) {
            if let Some(remote) = self.remote.as_mut() {
                remote.is_destroying = false;
            }
        }
        /* GAME_SEND */
        /* REMOTE_MINING */
    }
}
use remote::Remote;
struct Game<'a> {
    window: bool,
    lan_acting: Option<()>,
    lan_acting_uuid: Option<()>,
    player: Option<Player>,
    inventory: Vec<Option<ItemStack>>,
    selected: usize,
    res: Resources,
    world: World,
    remote: Option<Remote>,
    clicks: Clicks,
    creative: bool,
    mining: Option<Mining>,
    sequence: Cell<i32>,
    per_tick: f32,
    predicted: Vec<[i32; 3]>,
    marker: PhantomData<&'a ()>,
}
impl Game<'_> {
    fn native_window(&self) -> bool {
        self.window
    }
    fn client_hotbar_slot(&self) -> usize {
        self.selected
    }
    fn items(&self) -> &Items {
        &self.res.items
    }
    fn held(&self) -> Option<&ItemStack> {
        self.inventory.get(self.selected).and_then(Option::as_ref)
    }
    fn remote_send_carried(&self) {}
    fn native_publish_hand_derp_options(&self) {}
    fn pick(&self) -> Option<Hit> {
        Some(Hit {
            pos: [3, 42, -5],
            face: [0, 1, 0],
        })
    }
    fn remote_break_restricted(&self, _: [i32; 3]) -> bool {
        false
    }
    fn next_sequence(&self) -> i32 {
        let n = self.sequence.get() + 1;
        self.sequence.set(n);
        n
    }
    fn predict_break(&mut self, pos: [i32; 3], _: u32, _: i32) {
        self.predicted.push(pos);
    }
    fn destroy_progress(&self, _: [i32; 3]) -> f32 {
        self.per_tick
    }
    fn main_hand_key(&self) -> String {
        self.held()
            .map_or(String::new(), |stack| stack.item.to_string())
    }
    fn tutorial_destroy(&self, _: u32, _: f32) {}
    fn redstone_ore_particles(&self, _: [i32; 3]) {}
    fn breaking_block_effects(&self, _: [i32; 3], _: [i32; 3], _: bool) {}
}
fn face_direction(face: [i32; 3]) -> u8 {
    if face == [0, 1, 0] { 1 } else { 0 }
}
mod native_hooks {
    use super::*;
    impl Game<'_> {
        /* HELPERS */
    }
}
mod mod_api_host {
    include!(concat!(env!("OUT_DIR"), "/mod_api_host_generated.rs"));
}
fn game() -> (Game<'static>, Connection) {
    let (client, server) = Pipe::pair();
    let (_, writer) = Connection::local(client).split();
    let oracle: serde_json::Value = serde_json::from_str(include_str!("oracle.json")).unwrap();
    let item_tags = oracle["tags"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(tag, values)| {
            (
                tag.clone(),
                values
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_owned())
                    .collect(),
            )
        })
        .collect();
    (
        Game {
            window: true,
            lan_acting: None,
            lan_acting_uuid: None,
            player: Some(Player {
                server_copy: false,
                replaying: false,
            }),
            inventory: vec![Some(ItemStack {
                item: "minecraft:diamond_pickaxe".into(),
                count: 1,
            })],
            selected: 0,
            res: Resources {
                items: Items { item_tags },
            },
            world: World,
            remote: Some(remote::remote(Session {
                writer: Arc::new(Mutex::new(Some(writer))),
                configuring: Arc::new(AtomicBool::new(false)),
                serial: 17,
                debug: false,
            })),
            clicks: Clicks::default(),
            creative: false,
            mining: None,
            sequence: Cell::new(0),
            per_tick: 0.01,
            predicted: Vec::new(),
            marker: PhantomData,
        },
        Connection::local(server),
    )
}
fn receive(server: &mut Connection) -> S {
    let (id, bytes) = server.recv().unwrap();
    S::decode(id, &bytes).unwrap()
}
fn stop(pos: [i32; 3], direction: u8) -> S {
    S::PlayerAction(play::PlayerAction {
        action: 3,
        pos,
        direction,
        sequence: 91,
    })
}
#[test]
fn original_kotlin_real_item_tags_and_packet_constructor_results() {
    let (mut game, _) = game();
    let oracle: serde_json::Value = serde_json::from_str(include_str!("oracle.json")).unwrap();
    let items = oracle["items"].as_array().unwrap();
    assert!(items.len() > 1500);
    for item in items {
        assert!(!item["toolNegative"].as_bool().unwrap());
        for (count, key) in [(1, "tool"), (0, "toolEmpty")] {
            game.inventory[0] = Some(ItemStack {
                item: item["id"].as_str().unwrap().into(),
                count,
            });
            assert_eq!(
                game.client_fast_break_mining_tool(),
                item[key].as_bool().unwrap(),
                "{}",
                item["id"]
            );
        }
    }
    let rows = include_str!("commands.txt")
        .lines()
        .zip(include_str!("results.txt").lines());
    for (index, (line, result)) in rows.enumerate() {
        let t: Vec<_> = line.split(' ').collect();
        let expected: serde_json::Value = serde_json::from_str(result).unwrap();
        let behavior = fast_break::FastBreak {
            break_damage: t[1].parse().unwrap(),
            only_tool: t[2] == "true",
            mode: if t[3] == "true" {
                fast_break::Mode::Grim2348
            } else {
                fast_break::Mode::None
            },
        };
        game.inventory[0] = Some(ItemStack {
            item: t[0].into(),
            count: 1,
        });
        let (mut delay, mut progress) = (t[5].parse().unwrap(), t[6].parse().unwrap());
        let running = t[4] == "true";
        let tool = game.client_fast_break_mining_tool();
        behavior.tick(running, tool, &mut delay, &mut progress);
        assert_eq!(
            delay,
            expected["delay"].as_i64().unwrap() as i32,
            "row {index}"
        );
        assert_eq!(
            progress.to_bits(),
            expected["progressBits"].as_u64().unwrap() as u32,
            "row {index}: {line}"
        );
        let position =
            behavior.abort_above(running, tool, t[7] == "3", [3, t[9].parse().unwrap(), -5]);
        assert_eq!(
            position.map(|v| serde_json::json!(v)),
            if expected["abort"].is_null() {
                None
            } else {
                Some(expected["abort"]["pos"].clone())
            },
            "row {index}"
        );
        if let Some(abort) = expected["abort"].as_object() {
            assert_eq!(abort["direction"], t[8].parse::<i32>().unwrap());
            assert_eq!(abort["sequence"], 0);
        }
        assert_eq!(
            expected["disabledDelay"],
            fast_break::FastBreak::disabled_delay()
        );
    }
}
#[test]
fn generated_host_writes_real_client_game_mode_and_actual_remote_mining_finishes_early() {
    use mod_api::Host;
    let (mut game, mut server) = game();
    game.client_fast_break_update(true, 0.8, true);
    assert_eq!(game.remote.as_ref().unwrap().destroy_delay, 0);
    assert_eq!(game.remote.as_ref().unwrap().destroying.unwrap().2, 1.0);
    remote::mine(&mut game, false);
    assert_eq!(game.predicted, [[3, 42, -5]]);
    assert_eq!(game.remote.as_ref().unwrap().destroy_delay, 5);
    assert_eq!(
        receive(&mut server),
        S::PlayerAction(play::PlayerAction {
            action: 3,
            pos: [3, 42, -5],
            direction: 1,
            sequence: 1
        })
    );
    game.client_fast_break_update(true, 0.8, true);
    assert_eq!(game.remote.as_ref().unwrap().destroy_delay, 0);
    game.client_fast_break_reset();
    assert_eq!(game.remote.as_ref().unwrap().destroy_delay, 5);
    game.remote = None;
    game.clicks.destroying = LocalDestroying {
        delay: 4,
        progress: 0.9,
    };
    game.client_fast_break_update(true, 0.8, true);
    assert_eq!(
        (
            game.clicks.destroying.delay,
            game.clicks.destroying.progress
        ),
        (0, 1.0)
    );
    game.client_fast_break_reset();
    assert_eq!(game.clicks.destroying.delay, 5);
}
#[test]
fn native_identity_gates_preserve_remote_and_local_game_mode_state() {
    use mod_api::Host;
    for gate in 0..7 {
        let (mut game, _) = game();
        match gate {
            0 => game.window = false,
            1 => game.lan_acting = Some(()),
            2 => game.lan_acting_uuid = Some(()),
            3 => game.player = None,
            4 => game.player.as_mut().unwrap().server_copy = true,
            5 => game.player.as_mut().unwrap().replaying = true,
            6 => {
                game.inventory[0] = Some(ItemStack {
                    item: "minecraft:shears".into(),
                    count: 1,
                })
            }
            _ => unreachable!(),
        }
        game.client_fast_break_update(true, 0.8, true);
        assert_eq!(
            game.remote.as_ref().unwrap().destroy_delay,
            4,
            "gate {gate}"
        );
        assert_eq!(
            game.remote.as_ref().unwrap().destroying.unwrap().2,
            0.85,
            "gate {gate}"
        );
    }
    let (mut game, _) = game();
    game.remote.as_mut().unwrap().destroy_delay = u32::MAX;
    game.client_fast_break_update(false, 0.8, false);
    assert_eq!(game.remote.as_ref().unwrap().destroy_delay, u32::MAX);
    game.inventory[0].as_mut().unwrap().count = 0;
    game.client_fast_break_update(true, 0.8, true);
    assert_eq!(game.remote.as_ref().unwrap().destroy_delay, u32::MAX);
    game.client_fast_break_reset();
    assert_eq!(game.remote.as_ref().unwrap().destroy_delay, 5);
}
#[test]
fn actual_pipeline_nested_abort_keeps_order_cancel_semantics_codec_and_session_guards() {
    struct CancelStop;
    impl packets::PacketListener for CancelStop {
        fn outgoing(&self, event: &mut packets::PacketEvent<'_, S>) {
            if matches!(event.packet,S::PlayerAction(a) if a.action==3) {
                event.cancel();
            }
        }
    }
    let (mut game, mut server) = game();
    let service = Arc::new(fast_break_packets::FastBreakService::default());
    service.set_running(true);
    *service.behavior.lock().unwrap() = fast_break::FastBreak {
        mode: fast_break::Mode::Grim2348,
        only_tool: true,
        ..Default::default()
    };
    let pipeline = Arc::new(packets::PacketPipeline::default());
    pipeline.add_listener(0, service.clone());
    let previous = packet_bridge::set_interceptor(Some(pipeline.clone()));
    game.send(stop([3, 42, -5], 5));
    assert_eq!(
        receive(&mut server),
        S::PlayerAction(play::PlayerAction {
            action: 2,
            pos: [3, 43, -5],
            direction: 5,
            sequence: 0
        })
    );
    assert_eq!(receive(&mut server), stop([3, 42, -5], 5));
    let cancel = pipeline.add_listener(1, Arc::new(CancelStop));
    game.send(stop([3, 42, -5], 2));
    game.send(S::ClientTickEnd(Empty));
    assert_eq!(
        receive(&mut server),
        S::PlayerAction(play::PlayerAction {
            action: 2,
            pos: [3, 43, -5],
            direction: 2,
            sequence: 0
        })
    );
    assert_eq!(receive(&mut server), S::ClientTickEnd(Empty));
    pipeline.remove_listener(cancel);
    // The normal send refreshes the actual selected item even within a tick.
    game.inventory[0] = Some(ItemStack {
        item: "minecraft:diamond_sword".into(),
        count: 1,
    });
    game.send(stop([3, 42, -5], 1));
    assert_eq!(receive(&mut server), stop([3, 42, -5], 1));
    game.inventory[0] = Some(ItemStack {
        item: "minecraft:diamond_hoe".into(),
        count: 1,
    });
    game.send(stop([3, 42, -5], 1));
    assert!(matches!(receive(&mut server),S::PlayerAction(a) if a.action==2));
    assert_eq!(receive(&mut server), stop([3, 42, -5], 1));
    let session = &game.remote.as_ref().unwrap().session;
    session.send_silently(stop([3, 42, -5], 1));
    assert_eq!(receive(&mut server), stop([3, 42, -5], 1));
    assert!(fast_break_bridge::snapshot(18).is_none());
    fast_break_bridge::publish(Some(fast_break_bridge::Snapshot {
        session: 18,
        mining_tool: true,
    }));
    session.send(stop([3, 42, -5], 1));
    assert_eq!(receive(&mut server), stop([3, 42, -5], 1));
    service.set_running(false);
    game.send(stop([3, 42, -5], 1));
    assert_eq!(receive(&mut server), stop([3, 42, -5], 1));
    packet_bridge::set_interceptor(previous);
    fast_break_bridge::publish(None);
}
