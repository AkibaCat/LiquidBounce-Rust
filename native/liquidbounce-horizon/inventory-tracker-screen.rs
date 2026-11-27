// SPDX-License-Identifier: GPL-3.0-or-later
//! ViewedInventoryScreen is a read-only Screen, not a network container menu.
use super::{Game, inventory_tracker_native::SCREEN};
use horizon_client::{
    hud::{HudBatch, rgba},
    text::Text,
};
use serde_json::{Value, json};
use winit::keyboard::KeyCode;

impl Game<'_> {
    fn client_inventory_tracker_item(
        &self,
        batch: &mut HudBatch,
        stack: &super::ItemStack,
        x: f32,
        y: f32,
        seed: i32,
    ) {
        let owner = self.player_owner(Some(stack));
        let state = self.resolve_item(
            stack,
            horizon_assets::item_model::DisplayContext::Gui,
            Some(&owner),
            seed,
        );
        horizon_client::item_render::gui::draw(
            &state,
            &self.pack.item_renderer,
            &self.gui.atlas.sprites,
            &self.item_host(),
            batch,
            x,
            y,
            self.glint_frame.millis / 1000.,
            self.glint_frame.speed,
            self.glint_frame.alpha,
        );
    }
    /// Uses the live chat's native pending suggestions, with the same Tab,
    /// click, narration and input range handling as ordinary completions.
    pub(super) fn client_inventory_tracker_complete(&mut self) {
        self.client_inventory_tracker_world_refresh();
        let Some(screen) = self.ui.chat_screen.as_ref() else {
            self.client_inventory_tracker.completion_signature = None;
            self.client_inventory_tracker.completion_owned = false;
            return;
        };
        let input =
            json!({"kind":"completion","text":screen.input.value,"cursor":screen.input.cursor,
            "online":self.client_anti_bot_online()})
            .to_string();
        let mut output = String::new();
        self.native_client_inventory_tracker(&input, &mut output);
        let signature = format!("{input}\0{output}");
        if self.client_inventory_tracker.completion_signature.as_ref() == Some(&signature) {
            return;
        }
        self.client_inventory_tracker.completion_signature = Some(signature);
        let v = serde_json::from_str::<Value>(&output).unwrap_or(Value::Null);
        let handled = v["handled"] == true;
        let previous = self.client_inventory_tracker.completion_owned;
        self.client_inventory_tracker.completion_owned = handled;
        if !handled && !previous {
            return;
        }
        let range = if handled {
            let Some(start) = v["range"][0].as_u64().and_then(|n| usize::try_from(n).ok()) else {
                return;
            };
            let Some(len) = v["range"][1].as_u64().and_then(|n| usize::try_from(n).ok()) else {
                return;
            };
            let Some(end) = start.checked_add(len) else {
                return;
            };
            (start, end)
        } else {
            (0, 0)
        };
        let suggestions = if handled {
            v["suggestions"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|s| s.as_str().map(str::to_owned))
                .collect()
        } else {
            Vec::new()
        };
        let measure = horizon_client::screen_draw::Measure(&self.gui.atlas.font);
        if let Some(screen) = self.ui.chat_screen.as_mut() {
            screen.liquidbounce_inventory_suggestions(&measure, range, suggestions);
        }
    }
    pub(super) fn client_inventory_tracker_key(&mut self, raw: KeyCode) -> bool {
        if self.native_mod_screen() != Some(SCREEN) {
            return false;
        }
        if raw == KeyCode::Escape
            || self
                .settings
                .keys
                .translate_all(raw)
                .contains(&Ok(KeyCode::KeyE))
        {
            self.native_mod_screen_close(SCREEN);
        }
        // ViewedInventoryScreen.keyPressed always returns true.
        true
    }
    pub(super) fn client_inventory_tracker_lore(&mut self, lines: &mut Vec<(Text, u32)>) {
        if self.native_mod_screen() != Some(SCREEN) {
            return;
        }
        let Some(handle) = self.client_inventory_tracker.tooltip else {
            return;
        };
        let input =
            json!({"kind":"lore","handle":handle,"time":Self::client_inventory_tracker_time()})
                .to_string();
        let mut output = String::new();
        self.native_client_inventory_tracker(&input, &mut output);
        let v = serde_json::from_str::<Value>(&output).unwrap_or(Value::Null);
        if let Some(lore) = v["lore"].as_str() {
            lines.push((Text::literal(lore), rgba(170, 170, 170, 255)));
        }
    }
    pub(super) fn client_inventory_tracker_draw(&mut self, batch: &mut HudBatch) {
        if self.native_mod_screen() != Some(SCREEN) {
            return;
        }
        self.client_inventory_tracker_world_refresh();
        let Some(p) = self
            .client_inventory_tracker
            .viewed
            .as_deref()
            .and_then(|uuid| self.client_inventory_tracker.resolve(uuid))
            .cloned()
        else {
            return;
        };
        let (ox, oy) = (
            ((batch.width - 176.) / 2.).floor(),
            ((batch.height - 166.) / 2.).floor(),
        );
        self.render_transparent_background("mod", batch);
        if let Some(mut r) = self
            .gui
            .atlas
            .sprites
            .get("gui/container/inventory")
            .copied()
        {
            let du = (r.u1 - r.u0) / r.w as f32;
            let dv = (r.v1 - r.v0) / r.h as f32;
            r.u1 = r.u0 + 176. * du;
            r.v1 = r.v0 + 166. * dv;
            batch.rect_uv(ox, oy, 176., 166., r, u32::MAX);
        }
        let local = self.remote.as_ref().map_or(0, |r| r.entity_id);
        if p.id == local && p.current {
            self.draw_player_preview(batch, ox + 26., oy + 8., ox + 75., oy + 78., 30., 0.0625);
        } else if let Some(model) = self.client_inventory_tracker_model(&p) {
            self.client_inventory_tracker_preview(batch, &model, &p.uuid, &p.name, ox, oy);
        }
        let mut slots = vec![(p.crafting[0], 154., 28., None)];
        for i in 0..4 {
            slots.push((
                p.crafting[i + 1],
                98. + 18. * (i % 2) as f32,
                18. + 18. * (i / 2) as f32,
                None,
            ));
        }
        for (i, icon) in ["helmet", "chestplate", "leggings", "boots"]
            .into_iter()
            .enumerate()
        {
            slots.push((p.equipment[5 - i], 8., 8. + 18. * i as f32, Some(icon)));
        }
        for i in 9..36 {
            slots.push((
                p.inventory[i],
                8. + 18. * ((i - 9) % 9) as f32,
                84. + 18. * ((i - 9) / 9) as f32,
                None,
            ));
        }
        for i in 0..9 {
            slots.push((p.inventory[i], 8. + 18. * i as f32, 142., None));
        }
        slots.push((p.equipment[1], 77., 62., Some("shield")));
        let mut hovered = None;
        for (handle, x, y, icon) in slots {
            if let Some(stack) = self.client_inventory_tracker.stack(handle) {
                self.client_inventory_tracker_item(
                    batch,
                    stack,
                    ox + x,
                    oy + y,
                    x as i32 + y as i32 * 176,
                );
                self.draw_item_decorations(batch, stack, ox + x, oy + y, None);
            } else if let Some(icon) = icon {
                batch.sprite(
                    &self.gui.atlas,
                    &format!("gui/sprites/container/slot/{icon}"),
                    ox + x,
                    oy + y,
                );
            }
            let (mx, my) = (self.cursor.0 - ox, self.cursor.1 - oy);
            if mx >= x - 1. && mx < x + 17. && my >= y - 1. && my < y + 17. {
                hovered = Some(handle);
                batch.fill(
                    &self.gui.atlas,
                    ox + x,
                    oy + y,
                    16.,
                    16.,
                    rgba(255, 255, 255, 128),
                );
            }
        }
        if let Some(stack) = self.client_inventory_tracker.stack(p.carried) {
            self.draw_item_model(batch, stack, self.cursor.0 - 8., self.cursor.1 - 8.);
            self.draw_item_decorations(batch, stack, self.cursor.0 - 8., self.cursor.1 - 8., None);
        } else if let Some((handle, stack)) = hovered.and_then(|id| {
            self.client_inventory_tracker
                .stack(id)
                .cloned()
                .map(|s| (id, s))
        }) {
            let prior = self.client_inventory_tracker.tooltip.replace(handle);
            self.draw_stack_tooltip(batch, &stack, self.cursor.0, self.cursor.1);
            self.client_inventory_tracker.tooltip = prior;
        }
    }
}
