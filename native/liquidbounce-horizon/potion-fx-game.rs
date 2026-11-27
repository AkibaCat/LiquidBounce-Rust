// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_client::effects::{EffectInstance, PlayerEffects};
use horizon_core::geometry::Vec3;
use horizon_protocol::packets::play::Clientbound;
use serde_json::{Value, json};
use std::{cell::RefCell, collections::BTreeMap};
#[derive(Default)]
pub(super) struct NativeState {
    effects: RefCell<BTreeMap<i32, PlayerEffects>>,
}
fn at(p: Vec3) -> [f64; 3] {
    [p.x, p.y, p.z]
}
fn color(effects: &PlayerEffects) -> Option<i32> {
    let visible = effects
        .list
        .iter()
        .filter(|e| e.visible)
        .collect::<Vec<_>>();
    (!visible.is_empty()).then(|| {
        (0xff000000
            | horizon_client_core::potions::effects_color(
                visible.into_iter().map(|e| (e.id, e.amplifier)),
            )) as i32
    })
}
impl Game<'_> {
    pub(super) fn client_potion_fx_reset(&self) {
        self.liquidbounce_potion_fx.effects.borrow_mut().clear();
    }
    pub(super) fn client_potion_fx_tick(&self) {
        for e in self
            .liquidbounce_potion_fx
            .effects
            .borrow_mut()
            .values_mut()
        {
            let _ = e.tick();
        }
    }
    pub(super) fn client_potion_fx_errors(&mut self) {
        if self.native_window() && self.native_client_slot_owner() {
            for error in horizon_client::gfx::potion_fx::take_errors() {
                horizon_client::mod_api::Host::chat(self, &error);
            }
        }
    }
    pub(super) fn client_potion_fx_packet(&self, packet: &Clientbound) {
        if !self.native_window() {
            return;
        }
        match packet {
            Clientbound::UpdateMobEffect(p) => {
                let Some(id) = self.remote_effect_id(p.effect) else {
                    return;
                };
                let mut effects = self.liquidbounce_potion_fx.effects.borrow_mut();
                let e = effects.entry(p.id).or_default();
                let mut value = EffectInstance::new(id, p.duration, p.amplifier);
                value.ambient = p.flags & 1 != 0;
                value.visible = p.flags & 2 != 0;
                value.show_icon = p.flags & 4 != 0;
                if let Some(existing) = e.list.iter_mut().find(|v| v.id == id) {
                    *existing = value;
                } else {
                    e.list.push(value);
                }
            }
            Clientbound::RemoveMobEffect(p) => {
                if let Some(id) = self.remote_effect_id(p.effect)
                    && let Some(e) = self
                        .liquidbounce_potion_fx
                        .effects
                        .borrow_mut()
                        .get_mut(&p.id)
                {
                    e.list.retain(|v| v.id != id);
                }
            }
            Clientbound::RemoveEntities(p) => {
                let mut e = self.liquidbounce_potion_fx.effects.borrow_mut();
                for id in &p.ids {
                    e.remove(id);
                }
            }
            Clientbound::LevelEvent(p) if p.kind == 2002 => {
                let [x, y, z] = p.pos;
                let min = [f64::from(x) - 3., f64::from(y) - 3., f64::from(z) - 3.];
                let max = [f64::from(x) + 4., f64::from(y) + 4., f64::from(z) + 4.];
                let nearby = self.potions.thrown_entities().iter().any(|(kind, p, _)| {
                    *kind == "minecraft:splash_potion"
                        && p.x + 0.125 > min[0]
                        && p.x - 0.125 < max[0]
                        && p.y + 0.25 > min[1]
                        && p.y < max[1]
                        && p.z + 0.125 > min[2]
                        && p.z - 0.125 < max[2]
                });
                if !nearby {
                    return;
                }
                let center = [f64::from(x) + 0.5, f64::from(y) + 0.5, f64::from(z) + 0.5];
                let position = self.client_potion_fx_ground(center).unwrap_or(center);
                self.native_client_potion_fx(
                    &json!({"action":"splash","color":p.data,"position":position}).to_string(),
                    &mut String::new(),
                );
            }
            _ => {}
        }
    }
    pub(super) fn client_potion_fx_players(&self) -> Vec<Value> {
        let mut out = vec![];
        if let Some(p) = &self.player
            && let Some(color) = color(&self.potions.effects)
        {
            out.push(json!({"position":at(p.position),"old":at(self.previous_position),"age":p.health.tick_count,"color":color}));
        }
        for g in self
            .lan
            .iter()
            .flat_map(|h| h.hub.guests.iter())
            .filter(|g| g.placed && g.dimension == self.dim.id())
        {
            if let Some(c) = color(&g.effects) {
                out.push(json!({"position":g.position,"old":g.previous_position,"age":g.motion.as_ref().map_or(g.effects.tick_count,|p|p.health.tick_count),"color":c}));
            }
        }
        let synced = self.liquidbounce_potion_fx.effects.borrow();
        for m in self.mobs.iter().filter(|m| {
            !m.removed && horizon_client::command::args::normalize(&m.kind.id) == "minecraft:player"
        }) {
            let c = if self.remote.is_some() {
                synced.get(&(m.id as i32)).and_then(color)
            } else {
                let visible = m
                    .extra
                    .effects
                    .iter()
                    .filter(|(id, _, _)| !m.extra.hidden_effects.contains(id))
                    .collect::<Vec<_>>();
                (!visible.is_empty()).then(|| {
                    (0xff000000
                        | horizon_client_core::potions::effects_color(
                            visible.iter().map(|&&(id, _, amp)| (id, amp)),
                        )) as i32
                })
            };
            if let Some(c) = c {
                out.push(json!({"position":at(m.position),"old":at(m.previous),"age":m.age as i32,"color":c}));
            }
        }
        out
    }
    pub(super) fn client_potion_fx_frame(&self, partial: f64) {
        let mut geometry = "null".to_owned();
        if self.native_window() {
            let orientation = self.camera_orientation();
            self.native_client_potion_fx(&json!({"action":"frame","partial":partial as f32,"players":self.client_potion_fx_players(),"clouds":self.client_potion_fx_clouds(),"right":(orientation*glam::Vec3::X).to_array(),"up":(orientation*glam::Vec3::Y).to_array()}).to_string(),&mut geometry);
        }
        horizon_client::gfx::potion_fx::begin_frame(&geometry);
    }
}
