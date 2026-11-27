// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleBetterInventory ae0eb396: all settings and the decisions made at the
//! real slot, item-cooldown and handled-screen render points.
#[path = "better_inventory_locale.rs"]
mod locale;

pub const HIGHLIGHT_MODES: [&str; 2] = ["Border", "Texture"];
pub const COOLDOWN_MODES: [&str; 3] = ["Percentage", "DurationTicks", "DurationSeconds"];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HighlightMode {
    #[default]
    Border,
    Texture,
}
impl HighlightMode {
    pub fn tag(self) -> &'static str {
        HIGHLIGHT_MODES[self as usize]
    }
    pub fn parse(tag: &str) -> Option<Self> {
        match tag {
            "Border" => Some(Self::Border),
            "Texture" => Some(Self::Texture),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CooldownMode {
    #[default]
    Percentage,
    DurationTicks,
    DurationSeconds,
}
impl CooldownMode {
    pub fn tag(self) -> &'static str {
        COOLDOWN_MODES[self as usize]
    }
    pub fn parse(tag: &str) -> Option<Self> {
        fn fold(c: char) -> char {
            let mut upper = c.to_uppercase();
            let first = upper.next().unwrap_or(c);
            let upper = if upper.next().is_none() { first } else { c };
            upper.to_lowercase().next().unwrap_or(upper)
        }
        [Self::Percentage, Self::DurationTicks, Self::DurationSeconds]
            .into_iter()
            .find(|mode| mode.tag().chars().map(fold).eq(tag.chars().map(fold)))
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct BetterInventory {
    pub highlight_clicked: bool,
    pub highlight_mode: HighlightMode,
    /// Java Color4b.argb, with no premultiplication.
    pub highlight_color: i32,
    pub text_cooldown: bool,
    pub cooldown_mode: CooldownMode,
    pub cooldown_scale: f32,
    pub cooldown_color: i32,
    pub container_view: bool,
    pub skip_empty_stack: bool,
    pub container_scale: f32,
    pub relative_to_mouse: bool,
    pub render_offset_x: f32,
    pub render_offset_y: f32,
    pub move_cursor: bool,
}
impl Default for BetterInventory {
    fn default() -> Self {
        Self {
            highlight_clicked: true,
            highlight_mode: HighlightMode::Border,
            highlight_color: 0xff46_77ff_u32 as i32,
            text_cooldown: true,
            cooldown_mode: CooldownMode::Percentage,
            cooldown_scale: 1.0,
            cooldown_color: -1,
            container_view: true,
            skip_empty_stack: false,
            container_scale: 1.0,
            relative_to_mouse: true,
            render_offset_x: 150.0,
            render_offset_y: 0.0,
            move_cursor: true,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Highlight {
    Border { rects: [[i32; 4]; 4], color: i32 },
    Texture { rect: [i32; 4] },
}
#[derive(Clone, Debug, PartialEq)]
pub struct CooldownText {
    pub text: String,
    pub center_x: i32,
    pub y: i32,
    pub scale: f32,
    pub color: i32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ContainerView {
    /// Indices into the actual component's itemCopies, after optional filtering.
    pub indices: Vec<usize>,
    pub center: [f32; 2],
    pub scale: f32,
    pub dimensions: [i32; 2],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawError {
    MissingCooldownEntry,
    InvalidContainerScale,
}

impl BetterInventory {
    pub fn should_move_cursor(&self, running: bool) -> bool {
        running && self.move_cursor
    }

    pub fn highlight(
        &self,
        running: bool,
        last_clicked: i32,
        index: i32,
        x: i32,
        y: i32,
    ) -> Option<Highlight> {
        if !running || !self.highlight_clicked || index != last_clicked {
            return None;
        }
        Some(match self.highlight_mode {
            HighlightMode::Border => Highlight::Border {
                rects: [
                    [x, y, x.wrapping_add(16), y.wrapping_add(1)],
                    [
                        x,
                        y.wrapping_add(15),
                        x.wrapping_add(16),
                        y.wrapping_add(16),
                    ],
                    [x, y.wrapping_add(1), x.wrapping_add(1), y.wrapping_add(15)],
                    [
                        x.wrapping_add(15),
                        y.wrapping_add(1),
                        x.wrapping_add(16),
                        y.wrapping_add(15),
                    ],
                ],
                color: self.highlight_color,
            },
            HighlightMode::Texture => Highlight::Texture {
                rect: [x.wrapping_sub(3), y.wrapping_sub(3), 22, 21],
            },
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn cooldown(
        &self,
        running: bool,
        empty: bool,
        has_player: bool,
        progress: f32,
        entry: Option<(i32, i32)>,
        x: i32,
        y: i32,
    ) -> Result<Option<CooldownText>, DrawError> {
        self.cooldown_with_locale(running, empty, has_player, progress, entry, x, y, "")
    }

    #[allow(clippy::too_many_arguments)]
    pub fn cooldown_with_locale(
        &self,
        running: bool,
        empty: bool,
        has_player: bool,
        progress: f32,
        entry: Option<(i32, i32)>,
        x: i32,
        y: i32,
        format_locale: &str,
    ) -> Result<Option<CooldownText>, DrawError> {
        if !running
            || empty
            || !self.text_cooldown
            || !has_player
            || progress <= 0.0
            || progress.is_nan()
        {
            return Ok(None);
        }
        let text = match self.cooldown_mode {
            CooldownMode::Percentage => format!("{}%", (progress * 100.0) as i32),
            CooldownMode::DurationTicks => {
                let (end, current) = entry.ok_or(DrawError::MissingCooldownEntry)?;
                end.wrapping_sub(current).to_string()
            }
            CooldownMode::DurationSeconds => {
                let (end, current) = entry.ok_or(DrawError::MissingCooldownEntry)?;
                let seconds = end.wrapping_sub(current) as f32 * 0.05;
                if seconds > 1.0 {
                    format!("{}s", seconds as i32)
                } else {
                    // Java Formatter's fixed precision rounds decimal ties
                    // away from zero; Rust's default formatter rounds to even.
                    let rounded = (f64::from(seconds) * 10.0).round() / 10.0;
                    locale::localize(&format!("{rounded:.1}s"), format_locale)
                }
            }
        };
        Ok(Some(CooldownText {
            text,
            center_x: x.wrapping_add(8),
            y,
            scale: self.cooldown_scale,
            color: self.cooldown_color,
        }))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn container(
        &self,
        running: bool,
        empty: bool,
        component: Option<&[bool]>,
        x: i32,
        y: i32,
        mouse_x: i32,
        mouse_y: i32,
    ) -> Result<Option<ContainerView>, DrawError> {
        if !running || empty || !self.container_view {
            return Ok(None);
        }
        let Some(stacks) = component else {
            return Ok(None);
        };
        let indices: Vec<_> = stacks
            .iter()
            .enumerate()
            .filter_map(|(index, nonempty)| (!self.skip_empty_stack || *nonempty).then_some(index))
            .collect();
        if indices.is_empty() {
            return Ok(None);
        }
        let mut center = [
            self.render_offset_x - x as f32,
            self.render_offset_y - y as f32,
        ];
        if self.relative_to_mouse {
            center[0] += mouse_x as f32;
            center[1] += mouse_y as f32;
        }
        if self.container_scale <= 0.0 || self.container_scale.is_nan() {
            return Err(DrawError::InvalidContainerScale);
        }
        let n = indices.len() as i32;
        Ok(Some(ContainerView {
            indices,
            center,
            scale: self.container_scale,
            dimensions: [18 * n.min(9), 18 * (n / 9 + i32::from(n % 9 != 0))],
        }))
    }

    /// MixinAbstractContainerScreen.extractSlots AFTER has this outer gate,
    /// so a carried container is deliberately never previewed by this hook.
    pub fn screen_preview_eligible(carried_empty: bool, has_hovered_slot: bool) -> bool {
        carried_empty && has_hovered_slot
    }
}
