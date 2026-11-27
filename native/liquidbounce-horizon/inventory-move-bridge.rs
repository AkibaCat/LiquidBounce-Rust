// SPDX-License-Identifier: GPL-3.0-or-later
use crate::inventory_move_model::{Screen, Settings, State};
use horizon_protocol::packets::play::Serverbound as S;
use std::sync::{
    Mutex, OnceLock,
    atomic::{AtomicBool, Ordering},
};
#[derive(Default)]
struct Bridge {
    model: State,
    delayed: Vec<(u64, S)>,
    scheduled: Vec<(u64, S)>,
    start_notice: bool,
}
fn state() -> &'static Mutex<Bridge> {
    static STATE: OnceLock<Mutex<Bridge>> = OnceLock::new();
    STATE.get_or_init(Default::default)
}
pub static SEARCH_FOCUSED: AtomicBool = AtomicBool::new(false);
pub fn configure(running: bool, v: [i32; 11], server_open: bool) {
    let s = Settings {
        behavior: v[0] as u8,
        passthrough_sneak: v[1] != 0,
        sprint_control: v[2] != 0,
        client_sprint: v[3] as u8,
        server_sprint: v[4] as u8,
        sneak_control: v[5] != 0,
        client_sneak: v[6] as u8,
        timer: v[7] != 0,
        speed: f32::from_bits(v[8] as u32),
        blink: v[9] != 0,
        maximum_time: v[10],
    };
    let mut b = state().lock().expect("InventoryMove");
    if b.model.running && !running {
        b.delayed.clear();
    } // onDisabled clears only unscheduled work
    b.model.configure(running, s, server_open);
}
pub fn observe(mut screen: Screen, serial: u64, now: i64) {
    screen.editing |= SEARCH_FOCUSED.load(Ordering::Acquire);
    let mut b = state().lock().expect("InventoryMove");
    if b.model.observe(screen, serial, now) {
        b.start_notice = true;
    }
}
pub fn snapshot() -> (Screen, bool, bool, f32, bool, i32) {
    let mut b = state().lock().expect("InventoryMove");
    let m = &b.model;
    let out = (
        m.screen,
        m.running,
        m.settings.timer,
        m.settings.speed,
        b.start_notice,
        m.settings.maximum_time,
    );
    b.start_notice = false;
    out
}
pub fn keyboard(key: usize, pressed: bool) -> bool {
    state()
        .lock()
        .expect("InventoryMove")
        .model
        .keyboard(key, pressed)
}
pub fn handles(key: usize) -> bool {
    let b = state().lock().expect("InventoryMove");
    b.model
        .settings
        .handles(b.model.running, b.model.screen, key)
}
pub fn safe_click(container: i32) -> bool {
    let b = state().lock().expect("InventoryMove");
    let m = &b.model;
    m.settings
        .safe_click(m.running, m.screen, container, m.pressed)
}
pub fn sprint(source: u8, moving: bool, old: bool) -> bool {
    let b = state().lock().expect("InventoryMove");
    let m = &b.model;
    m.settings
        .sprint(m.running, m.screen, m.server_open, source, moving, old)
}
pub fn movement_sprint(moving: bool, old: i8) -> i8 {
    let b = state().lock().expect("InventoryMove");
    let m = &b.model;
    if !m.running
        || !m.settings.sprint_control
        || !Settings::handled_screen(m.screen, m.server_open)
    {
        return old;
    }
    match m.settings.client_sprint {
        1 if moving => 1,
        2 => 0,
        _ => old,
    }
}
pub fn sneak(old: bool) -> bool {
    let b = state().lock().expect("InventoryMove");
    let m = &b.model;
    m.settings.sneak(m.running, m.screen, m.server_open, old)
}
pub fn expired(now: i64) -> bool {
    state().lock().expect("InventoryMove").model.expired(now)
}
pub fn container_packet(p: &S) -> bool {
    matches!(
        p,
        S::ContainerClick(_)
            | S::ContainerButtonClick(_)
            | S::SetCreativeModeSlot(_)
            | S::ContainerSlotStateChanged(_)
            | S::ContainerClose(_)
    )
}
pub fn blink(container: bool) -> u8 {
    state()
        .lock()
        .expect("InventoryMove")
        .model
        .blink(container)
}
pub fn delay(session: u64, packet: &S) -> bool {
    if !container_packet(packet) {
        return false;
    }
    let mut b = state().lock().expect("InventoryMove");
    if !b.model.stop_packet() {
        return false;
    }
    b.delayed.push((session, packet.clone()));
    true
}
/// Invoked once at the end of MovementInputEvent. Scheduled packets run at the next idle turn.
pub fn final_input(mut input: [bool; 7]) -> [bool; 7] {
    let mut b = state().lock().expect("InventoryMove");
    if b.model.running && !b.delayed.is_empty() {
        let pending = std::mem::take(&mut b.delayed);
        b.scheduled.extend(pending);
        input[..6].fill(false);
    }
    b.model.input = input;
    input
}
pub fn next_loop() -> Vec<(u64, S)> {
    std::mem::take(&mut state().lock().expect("InventoryMove").scheduled)
}
pub fn clear() {
    let mut b = state().lock().expect("InventoryMove");
    b.delayed.clear();
    b.scheduled.clear();
    b.model.input.fill(false);
    b.model.serial = None;
    b.model.screen = Screen::default();
    b.start_notice = false;
}
