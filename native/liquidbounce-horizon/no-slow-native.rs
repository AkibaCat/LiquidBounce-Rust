// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use horizon_client::no_slow_bridge as bridge;
use horizon_protocol::packets::play::{self, Serverbound as S};
impl Game<'_> {
    fn no_slow_action(&self, stack: Option<&ItemStack>) -> u8 {
        let Some(stack) = stack else {
            return 0;
        };
        if let Some(cons) = self.consumable(stack) {
            use horizon_world::consumable::UseAnimation as A;
            return match cons.animation {
                A::None => 0,
                A::Eat => 1,
                A::Drink => 2,
                A::Block => 3,
                A::Spyglass => 4,
                A::TootHorn => 5,
                A::Brush => 6,
                A::Bow => 7,
                A::Trident => 8,
                A::Crossbow => 9,
                A::Bundle => 10,
                A::Spear => 11,
            };
        }
        if self.blocks_attacks(stack).is_some() {
            return 3;
        }
        match &*stack.item {
            "minecraft:spyglass" => 4,
            "minecraft:goat_horn" => 5,
            "minecraft:brush" => 6,
            "minecraft:bow" => 7,
            "minecraft:trident" => 8,
            "minecraft:crossbow" => 9,
            name if name.ends_with("bundle") => 10,
            name if horizon_client::spear::Kinetic::of(name).is_some() => 11,
            _ => 0,
        }
    }
    pub(super) fn no_slow_context(&self) -> Option<[f64; 15]> {
        if !self.native_client_slot_owner() {
            return None;
        }
        let p = self.player.as_ref()?;
        let used = self.using.and_then(|(slot, _)| self.stack_in(slot));
        let action = self.no_slow_action(used);
        let ticks = self.using.map_or(0, |(_, t)| t as i32);
        let block = self
            .using
            .and_then(|(slot, t)| {
                let component = self.blocks_attacks(self.stack_in(slot)?)?;
                Some(t as i32 >= (component.delay_seconds * 20.).round() as i32)
            })
            .unwrap_or(false);
        Some([
            f64::from(u8::from(self.using.is_some())),
            f64::from(action),
            f64::from(u8::from(matches!(self.used_hand(), Some(Hand::Off)))),
            f64::from(ticks),
            f64::from((self.use_duration - ticks).max(0)),
            f64::from(u8::from(block)),
            f64::from(u8::from(p.move_vector != [0., 0.])),
            self.selected as f64,
            f64::from(u8::from(p.shift_key_down)),
            f64::from(
                self.no_slow_action(self.inventory.get(self.selected).and_then(Option::as_ref)),
            ),
            f64::from(self.no_slow_action(self.offhand.as_ref())),
            f64::from(u8::from(action == 3)),
            f64::from(self.yaw),
            f64::from(self.pitch),
            f64::from(u8::from(used.is_some_and(|s| self.consumable(s).is_some()))),
        ])
    }
    pub(super) fn no_slow_send(&mut self, text: &str) {
        if !self.native_client_slot_owner() {
            return;
        }
        let Ok(actions) = serde_json::from_str::<Vec<serde_json::Value>>(text) else {
            return;
        };
        for action in actions {
            let send = |g: &mut Self| {
                match action["kind"].as_str().unwrap_or("") {
                    "carried" => {
                        g.send(S::SetCarriedItem(play::SetCarriedItem {
                            slot: action["slot"].as_i64().unwrap_or(0) as i16,
                        }));
                    }
                    "use" => {
                        if g.remote.is_none() {
                            return;
                        }
                        let sequence = if action["predict"] == true {
                            g.next_sequence()
                        } else {
                            0
                        };
                        g.send(S::UseItem(play::UseItem {
                            hand: action["hand"].as_i64().unwrap_or(0) as i32,
                            sequence,
                            yaw: g.yaw,
                            pitch: g.pitch,
                        }));
                    }
                    "release" => {
                        let pos = if action["player"] == true {
                            g.player
                                .as_ref()
                                .map(|p| {
                                    [
                                        p.position.x.floor() as i32,
                                        p.position.y.floor() as i32,
                                        p.position.z.floor() as i32,
                                    ]
                                })
                                .unwrap_or([0; 3])
                        } else {
                            [0; 3]
                        };
                        g.send(S::PlayerAction(play::PlayerAction {
                            action: 5,
                            pos,
                            direction: if action["player"] == true { 1 } else { 0 },
                            sequence: 0,
                        }));
                    }
                    "stop" => {
                        if let Some((slot, ticks)) = g.using
                            && let Some(stack) = g.stack_in(slot).cloned()
                        {
                            g.release_used_item(&stack, ticks, false);
                        }
                    }
                    // The pinned target has no ViaFabricPlus connection. Original helpers
                    // intentionally emit nothing without it; the mode still shows its warning.
                    "legacySneak" => {}
                    _ => {}
                }
            };
            if action["untracked"] == true {
                bridge::without_tracking(|| send(self));
            } else {
                send(self);
            }
        }
    }
    pub(super) fn no_slow_slowness(&mut self, multiplier: f32) {
        if !self.native_client_slot_owner() {
            return;
        }
        let Some(level) = self.potions.effects.amplifier("slowness") else {
            return;
        };
        let Some(p) = self.player.as_mut() else {
            return;
        };
        use horizon_world::attributes::{Modifier, Operation};
        let Some(key) = p.attributes.key("minecraft:movement_speed") else {
            return;
        };
        let Ok(Some(handle)) = p.attributes.get_instance(Some(key)) else {
            return;
        };
        let Ok(modifier) = Modifier::new(
            "minecraft:effect.slowness",
            -f64::from(multiplier) * (f64::from(level) + 1.),
            Operation::AddMultipliedTotal,
        ) else {
            return;
        };
        let _ = p.attributes.edit(&handle, |value, observer| {
            value.add_or_update_transient(modifier, observer)
        });
    }
}
