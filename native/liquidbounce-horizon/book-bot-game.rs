// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_client::{item_data::WrittenBook, items::ItemStack, text::Style};
use horizon_protocol::packets::play::{self, Serverbound as S};
use serde_json::{Value, json};
impl Game<'_> {
    fn book_bot_candidate(&self, stack: Option<&ItemStack>) -> bool {
        stack.is_some_and(|s| {
            if s.count <= 0 || &*s.item != "minecraft:writable_book" {
                return false;
            }
            let view = horizon_client::item_render::stack_view(s);
            self.pack
                .item_renderer
                .models
                .components_view(&view)
                .get("minecraft:writable_book_content")
                .is_some_and(|c| {
                    c.get("pages")
                        .and_then(Value::as_array)
                        .is_none_or(Vec::is_empty)
                })
        })
    }
    pub(super) fn book_bot_selected(&self) -> i32 {
        self.selected as i32
    }
    pub(super) fn book_bot_snapshot(&self) -> Option<String> {
        if !self.native_window()
            || self.lan_acting.is_some()
            || self.lan_acting_uuid.is_some()
            || !self
                .player
                .as_ref()
                .is_some_and(|p| p.native_client_queries_allowed())
        {
            return None;
        }
        Some(json!({"selected":self.selected,"candidates":(0..41).map(|i|{
            if i==40{self.book_bot_candidate(self.offhand.as_ref())}else if i<36{self.book_bot_candidate(self.inventory.get(i).and_then(Option::as_ref))}else{self.book_bot_candidate(self.armor.get(39-i).and_then(Option::as_ref))}
        }).collect::<Vec<_>>()}).to_string())
    }
    pub(super) fn book_bot_write(&mut self, settings: &str, root: &str, count: i32) -> bool {
        if self.book_bot_snapshot().is_none()
            || !self.book_bot_candidate(self.inventory.get(self.selected).and_then(Option::as_ref))
        {
            return false;
        }
        let Ok(s) = serde_json::from_str::<Value>(settings) else {
            return false;
        };
        let mode = s["mode"].as_u64().unwrap_or(0);
        let pages = s["pages"][if mode == 0 { 0 } else { 1 }]
            .as_i64()
            .unwrap_or(50) as i32;
        let font = &self.gui.atlas.font;
        let style = Style::default();
        let width =
            |c: u16| font.advance(char::from_u32(u32::from(c)).unwrap_or('\u{fffd}'), &style);
        let content = if mode == 0 {
            let ascii = s["ascii"].as_bool().unwrap_or(false);
            let allow = s["space"].as_bool().unwrap_or(true);
            let (start, end) = if ascii {
                (0x21, 0x7e)
            } else {
                (0x800, 0x10ffff)
            };
            let generator = std::iter::repeat_with(|| {
                super::edge_random::random_i32_inclusive(start, end - 1) as u32
            })
            .filter(|c| allow || !super::book_bot_content::java_whitespace(*c));
            super::book_bot_content::build(generator, pages, width)
        } else {
            let source = std::path::PathBuf::from(s["source"].as_str().unwrap_or_default());
            let source = if source.is_absolute() {
                source
            } else {
                std::path::Path::new(root).join(source)
            };
            let chars = std::fs::read_to_string(source)
                .unwrap_or_default()
                .chars()
                .map(|c| c as u32)
                .collect::<Vec<_>>();
            // A cyclic file containing only discarded characters never advances a page.
            // Keep the game responsive in that degenerate source case.
            let cyclic = s["cyclic"].as_bool().unwrap_or(true);
            if cyclic && !chars.is_empty() && chars.iter().all(|c| matches!(*c as u16, 13 | 32)) {
                return false;
            }
            let generator = chars.iter().copied();
            if cyclic {
                super::book_bot_content::build(generator.cycle(), pages, width)
            } else {
                super::book_bot_content::build(generator, pages, width)
            }
        };
        let pages = content
            .iter()
            .map(|p| super::book_bot_content::wire_text(p))
            .collect::<Vec<_>>();
        let title = s["name"]
            .as_str()
            .unwrap_or("Generated book #%count%")
            .replace("%count%", &count.to_string());
        let author = self.own_profile_name();
        if let Some(stack) = self
            .inventory
            .get_mut(self.selected)
            .and_then(Option::as_mut)
        {
            stack.data_mut().written = Some(WrittenBook {
                title: title.clone(),
                author,
                generation: 0,
                pages: pages.clone(),
                page_tags: Vec::new(),
                resolved: true,
            });
        }
        self.send(S::EditBook(play::EditBook {
            slot: self.selected as i32,
            pages,
            title: s["sign"].as_bool().unwrap_or(true).then_some(title),
        }));
        true
    }
}
