// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use serde_json::{Value, json};
use std::{cell::RefCell, collections::BTreeMap};
#[derive(Default)]
struct PlayerLifetimes {
    next: u64,
    players: BTreeMap<(u8, i32), (i32, bool, u64)>,
}
thread_local! {static LIFETIMES:RefCell<PlayerLifetimes>=RefCell::new(PlayerLifetimes::default());}
fn lifetime(role: u8, id: i32, tick: i32, alive: bool) -> u64 {
    LIFETIMES.with_borrow_mut(|s| {
        let replace = s
            .players
            .get(&(role, id))
            .is_none_or(|&(last, was_alive, _)| tick < last || (!was_alive && alive));
        if replace {
            s.next += 1;
            let serial = s.next;
            s.players.insert((role, id), (tick, alive, serial));
        }
        let entry = s.players.get_mut(&(role, id)).unwrap();
        entry.0 = tick;
        entry.1 = alive;
        (entry.2 << 2) | u64::from(role)
    })
}
fn interpolate(a: [f64; 3], b: [f64; 3], alpha: f64) -> [f64; 3] {
    std::array::from_fn(|i| a[i] + (b[i] - a[i]) * alpha)
}
impl Game<'_> {
    pub(super) fn client_breadcrumbs_reset(&self) {
        LIFETIMES.with_borrow_mut(|s| s.players.clear());
        horizon_client::gfx::breadcrumbs::begin_frame("null");
    }
    fn client_breadcrumbs_entities(&self, alpha: f64) -> Vec<Value> {
        let mut entities = vec![];
        if let Some(p) = self.player.as_ref() {
            let at = [p.position.x, p.position.y, p.position.z];
            let previous = [
                self.previous_position.x,
                self.previous_position.y,
                self.previous_position.z,
            ];
            let alive = p.health.health() > 0.;
            let identity = lifetime(
                1,
                self.remote.as_ref().map_or(0, |r| r.entity_id),
                p.health.tick_count,
                alive,
            );
            entities.push(json!({"identity":identity,"own":true,"alive":alive,"position":at,"interpolated":interpolate(previous,at,alpha)}));
        }
        for m in self
            .mobs
            .iter()
            .filter(|m| &*m.kind.id == "minecraft:player" && !m.removed)
        {
            let at = [m.position.x, m.position.y, m.position.z];
            let previous = [m.previous.x, m.previous.y, m.previous.z];
            entities.push(json!({"identity":m.join.get()<<2,"own":false,"alive":m.health>0.,"position":at,"interpolated":interpolate(previous,at,alpha)}));
        }
        for g in self
            .lan
            .iter()
            .flat_map(|h| h.hub.guests.iter())
            .filter(|g| g.placed)
        {
            let tick = g
                .motion
                .as_ref()
                .map_or(g.ticks as i32, |m| m.health.tick_count);
            let alive = g.health > 0.;
            let identity = lifetime(2, g.entity_id, tick, alive);
            entities.push(json!({"identity":identity,"own":false,"alive":alive,"position":g.position,"interpolated":interpolate(g.previous_position,g.position,alpha)}));
        }
        entities
    }
    pub(super) fn client_breadcrumbs_tick(&self) {
        if !self.native_window() || self.player.is_none() {
            return;
        }
        let input =
            json!({"tick":true,"entities":self.client_breadcrumbs_entities(1.)}).to_string();
        self.native_client_breadcrumbs(&input, &mut String::new());
    }
    pub(super) fn client_breadcrumbs_frame(&self, alpha: f64) {
        let input =
            json!({"tick":false,"entities":self.client_breadcrumbs_entities(alpha)}).to_string();
        let mut geometry = "null".to_owned();
        self.native_client_breadcrumbs(&input, &mut geometry);
        horizon_client::gfx::breadcrumbs::begin_frame(&geometry);
    }
}
