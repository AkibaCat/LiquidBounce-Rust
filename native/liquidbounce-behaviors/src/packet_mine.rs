// SPDX-License-Identifier: GPL-3.0-or-later
// Source: LiquidBounce ae0eb396 world/packetmine. Copyright CCBlueX 2015-2026.
//! PacketMine state transitions. The caller supplies real block/ray/tool facts
//! and executes actions in order at the client event boundary.
pub type Pos = [i32; 3];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,
    Immediate,
    Civ,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolMode {
    Always,
    PostStart,
    #[default]
    OnStop,
    Never,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SwitchMethod {
    #[default]
    Normal,
    Swap,
    Pick,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Rotate {
    OnStart,
    OnStop,
    Both,
    Always,
    #[default]
    Never,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub mode: Mode,
    pub normal_client_set: bool,
    pub normal_confirm: bool,
    pub immediate_confirm: bool,
    pub civ_switch: bool,
    pub range: f32,
    pub walls: f32,
    pub keep: f32,
    pub swing: u8,
    pub tool: ToolMode,
    pub method: SwitchMethod,
    pub abort_on_switch: bool,
    pub cancel_automatic: bool,
    pub rotate: Rotate,
    pub ignore_inventory: bool,
    pub break_damage: f32,
    pub post_break_delay: i32,
    pub abort_down: bool,
    pub select_delay: i32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::Normal,
            normal_client_set: false,
            normal_confirm: true,
            immediate_confirm: true,
            civ_switch: false,
            range: 4.5,
            walls: 4.5,
            keep: 25.,
            swing: 1,
            tool: ToolMode::OnStop,
            method: SwitchMethod::Normal,
            abort_on_switch: true,
            cancel_automatic: true,
            rotate: Rotate::Never,
            ignore_inventory: true,
            break_damage: 1.,
            post_break_delay: 6,
            abort_down: false,
            select_delay: 200,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    pub pos: Pos,
    pub finished: bool,
    pub progress: f32,
    pub started: bool,
    pub finish_ready_tick: Option<i64>,
    pub direction: Option<u8>,
}
impl Target {
    pub fn new(pos: Pos) -> Self {
        Self {
            pos,
            finished: false,
            progress: 0.,
            started: false,
            finish_ready_tick: None,
            direction: None,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    AddTarget(Pos),
    RemoveTarget(Pos),
    ClearRenderer,
    /// START=0, ABORT=1, STOP=2. ABORT is queued into the tick executor.
    Dig {
        status: u8,
        pos: Pos,
        face: u8,
        queued: bool,
    },
    Swing(u8),
    Destroy(Pos),
    BreakingProgress(Pos),
    StopVanilla,
    SelectTool {
        slot: u8,
        delegate_auto_tool: bool,
    },
    SyncCarried,
    SwapTool(u8),
    PickTool(u8),
    HeldPacket(u8),
    UpdateBox {
        pos: Pos,
        scale: f32,
    },
    NoViaFabricPlus,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct BlockFacts {
    pub air: bool,
    pub hardness: f32,
    pub creative: bool,
    pub border: bool,
    pub distance_squared: f64,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct TickFacts {
    pub block: BlockFacts,
    pub current_hit: bool,
    pub direction: Option<u8>,
    pub delta: f32,
    pub slot: Option<u8>,
    pub selected: u8,
    pub auto_tool: bool,
    pub correct_tool_required: bool,
    pub correct_slot: Option<u8>,
    pub via_fabric_plus: bool,
}
#[derive(Clone, Debug)]
pub struct PacketMine {
    pub settings: Settings,
    pub target: Option<Target>,
    pub tick: i64,
    pub next_start: i64,
    pub last_select: i64,
}
impl Default for PacketMine {
    fn default() -> Self {
        Self::new(0)
    }
}
impl PacketMine {
    pub fn new(now: i64) -> Self {
        Self {
            settings: Default::default(),
            target: None,
            tick: 0,
            next_start: 0,
            last_select: now,
        }
    }
    fn can_abort(&self) -> bool {
        self.settings.mode != Mode::Immediate
    }
    fn abort(&self, t: &Target, forced: bool, outside: bool, out: &mut Vec<Action>) {
        if t.started && !t.finished && self.can_abort() && (forced || outside) {
            out.push(Action::Dig {
                status: 1,
                pos: t.pos,
                face: if self.settings.abort_down {
                    0
                } else {
                    t.direction.unwrap_or(0)
                },
                queued: true,
            });
        }
    }
    fn replace(&mut self, next: Option<Target>, out: &mut Vec<Action>) {
        if self.target.as_ref().map(|t| t.pos) == next.as_ref().map(|t| t.pos) {
            return;
        }
        if let Some(old) = self.target.take() {
            out.push(Action::RemoveTarget(old.pos));
            self.abort(&old, true, false, out);
        }
        if let Some(t) = next.as_ref() {
            out.push(Action::AddTarget(t.pos));
        }
        self.target = next;
    }
    pub fn enabled(&mut self) -> Vec<Action> {
        self.next_start = 0;
        vec![Action::StopVanilla]
    }
    pub fn disabled(&mut self) -> Vec<Action> {
        let mut out = vec![Action::ClearRenderer];
        self.next_start = 0;
        self.replace(None, &mut out);
        out
    }
    pub fn world_changed(&mut self) -> Vec<Action> {
        let mut out = vec![];
        self.replace(None, &mut out);
        out
    }
    fn valid_selection(&self, b: BlockFacts) -> bool {
        b.border
            && if self.settings.mode == Mode::Civ {
                b.hardness > 0.
            } else {
                b.hardness >= 0. && !b.air
            }
    }
    fn blocks_change(&self, t: &Target) -> bool {
        self.settings.mode == Mode::Normal && self.settings.normal_confirm && t.finished
    }
    /// Both physical left-button actions reach the original MouseButtonEvent.
    pub fn click(
        &mut self,
        pos: Pos,
        b: BlockFacts,
        now: i64,
        screen: bool,
        may_build: bool,
    ) -> Vec<Action> {
        let mut out = vec![];
        if screen
            || !may_build
            || self.settings.mode == Mode::Immediate && self.target.is_some()
            || now.wrapping_sub(self.last_select) < i64::from(self.settings.select_delay)
        {
            return out;
        }
        if self.target.as_ref().is_some_and(|t| self.blocks_change(t)) {
            self.last_select = now;
            return out;
        }
        let next = if self.valid_selection(b) && self.target.as_ref().is_none_or(|t| t.pos != pos) {
            Some(Target::new(pos))
        } else {
            None
        };
        self.replace(next, &mut out);
        self.last_select = now;
        out
    }
    pub fn set_target(&mut self, pos: Pos, b: BlockFacts) -> Vec<Action> {
        let mut out = vec![];
        if !self.valid_selection(b) || self.target.as_ref().is_some_and(|t| self.blocks_change(t)) {
            return out;
        }
        if self
            .target
            .as_ref()
            .is_none_or(|t| self.settings.mode != Mode::Immediate && t.finished)
        {
            self.replace(Some(Target::new(pos)), &mut out);
        }
        out
    }
    pub fn block_update(&mut self, pos: Pos, air: bool) -> Vec<Action> {
        let mut out = vec![];
        if air
            && self.settings.mode != Mode::Civ
            && self.target.as_ref().is_some_and(|t| t.pos == pos)
        {
            self.replace(None, &mut out);
        }
        out
    }
    pub fn held_slot_packet(&mut self, slot: u8, best: Option<u8>) -> Vec<Action> {
        let mut out = vec![];
        if self.settings.tool == ToolMode::Always
            && self.settings.abort_on_switch
            && self.target.as_ref().is_some_and(|t| t.started)
            && best == Some(slot)
        {
            self.replace(None, &mut out);
        }
        out
    }
    pub fn cancels_silent_select(
        &self,
        from_packet_mine: bool,
        from_auto_tool: bool,
        slot: u8,
        best: Option<u8>,
    ) -> bool {
        self.settings.tool == ToolMode::Always
            && self.settings.cancel_automatic
            && self.target.as_ref().is_some_and(|t| t.started)
            && !from_packet_mine
            && !(from_auto_tool && best == Some(slot))
    }
    pub fn should_rotate(&self) -> bool {
        self.target
            .as_ref()
            .is_some_and(|t| match self.settings.rotate {
                Rotate::OnStart => !t.started,
                Rotate::OnStop => t.progress >= self.settings.break_damage,
                Rotate::Both => !t.started || t.progress >= self.settings.break_damage,
                Rotate::Always => true,
                Rotate::Never => false,
            })
    }
    pub fn rotation_unreachable(&self, distance_squared: f64) -> Vec<Action> {
        let mut out = vec![];
        if let Some(t) = self.target.as_ref() {
            self.abort(
                t,
                false,
                distance_squared > f64::from(self.settings.keep).powi(2),
                &mut out,
            );
        }
        out
    }
    fn should_switch(&self, t: &Target) -> bool {
        match self.settings.tool {
            ToolMode::Always | ToolMode::PostStart => true,
            ToolMode::OnStop => t.progress >= self.settings.break_damage,
            ToolMode::Never => false,
        }
    }
    fn switch(&self, t: &Target, f: &TickFacts, out: &mut Vec<Action>) -> Option<u8> {
        if !self.should_switch(t) {
            return None;
        }
        let slot = f.slot?;
        match if self.settings.tool == ToolMode::OnStop {
            self.settings.method
        } else {
            SwitchMethod::Normal
        } {
            SwitchMethod::Normal => out.push(Action::SelectTool {
                slot,
                delegate_auto_tool: f.auto_tool,
            }),
            SwitchMethod::Swap => {
                if f.selected != slot {
                    out.push(Action::SwapTool(slot));
                    return Some(slot);
                }
            }
            SwitchMethod::Pick => {
                if f.via_fabric_plus {
                    out.push(Action::PickTool(slot));
                    return Some(slot);
                } else {
                    out.push(Action::NoViaFabricPlus);
                }
            }
        }
        None
    }
    fn sync_method(&self) -> bool {
        self.settings.tool != ToolMode::OnStop || self.settings.method == SwitchMethod::Normal
    }
    fn dig(&self, t: &Target, status: u8, out: &mut Vec<Action>) {
        out.push(Action::Dig {
            status,
            pos: t.pos,
            face: t.direction.unwrap_or(0),
            queued: false,
        });
        out.push(Action::Swing(self.settings.swing));
    }
    pub fn step(&mut self, f: TickFacts) -> Vec<Action> {
        self.tick = self.tick.wrapping_add(1);
        let mut out = vec![];
        let Some(mut t) = self.target.take() else {
            return out;
        };
        let invalid = if self.settings.mode == Mode::Civ {
            f.block.hardness == 1. && !f.block.creative
        } else {
            f.block.hardness < 0. && !f.block.creative || f.block.air
        };
        let outside = f.block.distance_squared > f64::from(self.settings.keep).powi(2);
        if invalid || outside {
            out.push(Action::RemoveTarget(t.pos));
            self.abort(&t, true, false, &mut out);
            return out;
        }
        if !f.current_hit {
            let needs = match self.settings.rotate {
                Rotate::OnStart => !t.started,
                Rotate::OnStop => t.progress >= self.settings.break_damage,
                Rotate::Both => !t.started || t.progress >= self.settings.break_damage,
                Rotate::Always => true,
                Rotate::Never => false,
            };
            if needs {
                if self.settings.mode == Mode::Civ {
                    out.push(Action::Dig {
                        status: 2,
                        pos: t.pos,
                        face: 0,
                        queued: false,
                    });
                }
                if self.settings.rotate == Rotate::Always && t.started {
                    self.abort(&t, false, outside, &mut out);
                } else {
                    self.switch(&t, &f, &mut out);
                    if self.sync_method() {
                        out.push(Action::SyncCarried);
                    }
                }
                self.target = Some(t);
                return out;
            }
        }
        let Some(face) = f.direction else {
            if self.settings.mode == Mode::Civ {
                out.push(Action::Dig {
                    status: 2,
                    pos: t.pos,
                    face: 0,
                    queued: false,
                });
            }
            self.abort(&t, false, outside, &mut out);
            self.target = Some(t);
            return out;
        };
        t.direction = Some(face);
        if f.block.creative {
            out.push(Action::Destroy(t.pos));
            self.dig(&t, 0, &mut out);
            self.target = Some(t);
            return out;
        }
        if !t.started {
            if self.tick >= self.next_start {
                self.switch(&t, &f, &mut out);
                if self.settings.tool == ToolMode::Always {
                    out.push(Action::SyncCarried);
                }
                out.push(Action::BreakingProgress(t.pos));
                self.dig(&t, 0, &mut out);
                if self.settings.mode == Mode::Immediate {
                    self.dig(&t, 2, &mut out);
                }
                t.started = true;
                t.finish_ready_tick = None;
            }
        } else if self.settings.mode == Mode::Civ && t.finished {
            let mut restore = None;
            if self.settings.civ_switch && f.correct_tool_required {
                if f.auto_tool {
                    out.push(Action::SelectTool {
                        slot: f.correct_slot.unwrap_or(f.selected),
                        delegate_auto_tool: true,
                    });
                } else if let Some(slot) = f.correct_slot.filter(|s| *s != f.selected) {
                    out.push(Action::HeldPacket(slot));
                    restore = Some(f.selected);
                }
            }
            out.push(Action::Dig {
                status: 2,
                pos: t.pos,
                face,
                queued: false,
            });
            if let Some(slot) = restore {
                out.push(Action::HeldPacket(slot));
            }
        } else if !t.finished
            && (self.settings.mode != Mode::Immediate || t.progress < self.settings.break_damage)
        {
            t.progress += f.delta;
            let exchanged = self.switch(&t, &f, &mut out);
            if self.sync_method() {
                out.push(Action::SyncCarried);
            }
            let scale = if self.settings.break_damage > 0. {
                t.progress.clamp(0., self.settings.break_damage) / self.settings.break_damage
            } else {
                1.
            };
            out.push(Action::UpdateBox { pos: t.pos, scale });
            if t.progress >= self.settings.break_damage {
                if let Some(ready) = t.finish_ready_tick {
                    if self.tick > ready {
                        self.next_start = self.tick + i64::from(self.settings.post_break_delay);
                        match self.settings.mode {
                            Mode::Normal => {
                                self.dig(&t, 2, &mut out);
                                if self.settings.normal_client_set {
                                    out.push(Action::Destroy(t.pos));
                                }
                                t.finished = true;
                            }
                            Mode::Immediate => {
                                if !self.settings.immediate_confirm {
                                    t.finished = true;
                                }
                            }
                            Mode::Civ => {
                                self.dig(&t, 2, &mut out);
                                t.finished = true;
                            }
                        }
                        if let Some(slot) = exchanged {
                            out.push(if self.settings.method == SwitchMethod::Swap {
                                Action::SwapTool(slot)
                            } else {
                                Action::PickTool(slot)
                            });
                        }
                        if t.finished
                            && match self.settings.mode {
                                Mode::Normal => !self.settings.normal_confirm,
                                Mode::Immediate => !self.settings.immediate_confirm,
                                Mode::Civ => false,
                            }
                        {
                            out.push(Action::RemoveTarget(t.pos));
                            return out;
                        }
                    }
                } else {
                    t.finish_ready_tick = Some(self.tick);
                }
            }
        }
        if out.iter().any(|a| *a == Action::NoViaFabricPlus) {
            out.push(Action::ClearRenderer);
            out.push(Action::RemoveTarget(t.pos));
            self.abort(&t, true, false, &mut out);
            self.next_start = 0;
            return out;
        }
        self.target = Some(t);
        out
    }
    pub fn progress(&self) -> Option<(Pos, f32)> {
        self.target.as_ref().filter(|t| t.started).map(|t| {
            (
                t.pos,
                if self.settings.break_damage > 0. {
                    (t.progress / self.settings.break_damage).clamp(0., 1.)
                } else {
                    1.
                },
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn facts() -> TickFacts {
        TickFacts {
            block: BlockFacts {
                hardness: 2.,
                border: true,
                ..Default::default()
            },
            current_hit: true,
            direction: Some(1),
            delta: 1.,
            ..Default::default()
        }
    }
    #[test]
    fn normal_delays_stop_until_tick_after_threshold_and_waits_for_update() {
        let mut m = PacketMine::default();
        m.set_target([1, 2, 3], facts().block);
        assert!(
            m.step(facts())
                .iter()
                .any(|a| matches!(a, Action::Dig { status: 0, .. }))
        );
        assert!(
            !m.step(facts())
                .iter()
                .any(|a| matches!(a, Action::Dig { status: 2, .. }))
        );
        assert!(
            m.step(facts())
                .iter()
                .any(|a| matches!(a, Action::Dig { status: 2, .. }))
        );
        assert!(m.target.as_ref().unwrap().finished);
        assert!(
            m.click([2, 2, 3], facts().block, 1000, false, true)
                .is_empty()
        );
        assert_eq!(m.target.as_ref().unwrap().pos, [1, 2, 3]);
        assert!(
            m.block_update([1, 2, 3], true)
                .contains(&Action::RemoveTarget([1, 2, 3]))
        );
    }
    #[test]
    fn immediate_sends_start_stop_but_never_aborts_or_manually_changes() {
        let mut m = PacketMine::default();
        m.settings.mode = Mode::Immediate;
        m.set_target([0; 3], facts().block);
        let a = m.step(facts());
        assert_eq!(
            a.iter().filter(|a| matches!(a, Action::Dig { .. })).count(),
            2
        );
        assert!(m.click([1; 3], facts().block, 1000, false, true).is_empty());
        assert!(
            !m.disabled()
                .iter()
                .any(|a| matches!(a, Action::Dig { status: 1, .. }))
        );
    }
    #[test]
    fn civ_keeps_air_target_and_spams_stop_after_finish() {
        let mut m = PacketMine::default();
        m.settings.mode = Mode::Civ;
        m.set_target([0; 3], facts().block);
        m.step(facts());
        m.step(facts());
        m.step(facts());
        let mut f = facts();
        f.block.air = true;
        f.block.hardness = 0.;
        let a = m.step(f);
        assert_eq!(
            a.iter()
                .filter(|a| matches!(a, Action::Dig { status: 2, .. }))
                .count(),
            1
        );
        assert!(m.block_update([0; 3], true).is_empty());
    }
    #[test]
    fn miss_pause_does_not_reset_progress_or_finish_early() {
        let mut m = PacketMine::default();
        m.settings.rotate = Rotate::Always;
        m.set_target([0; 3], facts().block);
        let mut f = facts();
        f.current_hit = false;
        assert!(
            !m.step(f)
                .iter()
                .any(|a| matches!(a, Action::Dig { status: 0, .. }))
        );
        m.step(facts());
        assert!(
            !m.step(f)
                .iter()
                .any(|a| matches!(a, Action::Dig { status: 1, .. }))
        );
        assert_eq!(m.target.as_ref().unwrap().progress, 0.);
    }
    #[test]
    fn abort_cleanup_range_and_button_delay_preserve_source_state() {
        let mut m = PacketMine::new(50);
        assert!(m.click([0; 3], facts().block, 249, false, true).is_empty());
        m.click([0; 3], facts().block, 250, false, true);
        m.step(facts());
        let a = m.click([0; 3], facts().block, 450, false, true);
        assert!(m.target.is_none());
        assert!(a.iter().any(|a| matches!(
            a,
            Action::Dig {
                status: 1,
                queued: true,
                ..
            }
        )));
    }
}
