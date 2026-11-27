// SPDX-License-Identifier: GPL-3.0-or-later
#[path = "liquidbounce_better_chat_layout.rs"]
mod liquidbounce_better_chat_layout;
impl Chat {
    pub fn liquidbounce_policy(&mut self, infinite: bool, anti_clear: bool) {
        self.liquidbounce_infinite = infinite;
        self.liquidbounce_anti_clear = anti_clear;
    }
    pub fn liquidbounce_force_clear(&mut self) {
        let paused = self.liquidbounce_anti_clear;
        self.liquidbounce_anti_clear = false;
        self.clear();
        self.liquidbounce_anti_clear = paused;
    }
    pub fn liquidbounce_add_stack(
        &mut self,
        atlas: &HudAtlas,
        mut content: Text,
        source: ChatSource,
        ticks: u64,
        focused: bool,
        id: String,
    ) {
        let count = self
            .messages
            .iter()
            .find(|m| m.liquidbounce_id.as_deref() == Some(&id))
            .map_or(1, |m| m.liquidbounce_count.saturating_add(1));
        let had = self
            .messages
            .iter()
            .any(|m| m.liquidbounce_id.as_deref() == Some(&id));
        if had {
            self.messages
                .retain(|m| m.liquidbounce_id.as_deref() != Some(&id));
            self.rewrap(&atlas.font);
        }
        if count > 1 {
            content = content.append(Text::literal(format!(" [{count}]")).with_color(text::GRAY));
        }
        self.push_message(
            &atlas.font,
            Message {
                added: ticks,
                time: (0, 0),
                repeats: 1,
                wrapped: 0,
                content,
                tag: None,
                source,
                signature: None,
                client_id: None,
                liquidbounce_id: Some(id),
                liquidbounce_count: count,
            },
            focused,
        );
    }
    fn liquidbounce_hover_bounds(&self, height: i32, x: f64, y: f64) -> Option<(usize, usize)> {
        let index = liquidbounce_better_chat_layout::hit(
            self.lines.len(),
            self.scroll,
            self.lines_per_page(true),
            self.options.width_px(),
            height,
            self.options.scale,
            self.options.line_height(),
            x,
            y,
        )?;
        let keys = self
            .lines
            .iter()
            .map(|line| std::sync::Arc::as_ptr(&line.message))
            .collect::<Vec<_>>();
        liquidbounce_better_chat_layout::bounds(&keys, index)
    }
    pub fn liquidbounce_copy_at(&self, height: i32, x: f64, y: f64) -> Option<String> {
        let (start, end) = self.liquidbounce_hover_bounds(height, x, y)?;
        let mut out = String::new();
        for line in self.lines[start..=end].iter().rev() {
            out.push_str(&line.content.plain());
        }
        Some(out)
    }
    pub fn liquidbounce_highlight(&self, batch: &mut HudBatch, atlas: &HudAtlas, x: f64, y: f64) {
        let Some(bounds) = self.liquidbounce_hover_bounds(batch.height as i32, x, y) else {
            return;
        };
        let Some([left, top, right, bottom]) = liquidbounce_better_chat_layout::rect(
            bounds,
            self.scroll,
            self.lines.len(),
            self.lines_per_page(true),
            self.options.width_px(),
            batch.height as i32,
            self.options.scale,
            self.options.line_height(),
        ) else {
            return;
        };
        hud_layout::paint(
            batch,
            atlas,
            &[hud_layout::fill(left, top, right, bottom, 0x4422aaff)],
        );
    }
}
