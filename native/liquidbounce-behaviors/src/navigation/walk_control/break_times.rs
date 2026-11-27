// SPDX-License-Identifier: GPL-3.0-or-later
//! Immutable copied-tool mining times from Agent BreakTimes.kt.
pub trait BreakState {
    /// getDestroySpeed(EmptyBlockGetter, BlockPos.ZERO).
    fn empty_level_hardness(&self) -> f32;
    fn requires_correct_tool(&self) -> bool;
}
pub trait BreakStack<B>: Clone + Send + Sync + 'static {
    fn is_empty(&self) -> bool;
    fn copy_stack(&self) -> Self;
    fn efficiency_level(&self) -> i32;
    fn correct_for_drops(&self, state: &B) -> bool;
    fn destroy_speed(&self, state: &B) -> f32;
    fn empty_hand() -> Self;
}
struct Tool<T> {
    stack: T,
    efficiency: i32,
}
pub struct BreakTimes<T> {
    tools: Vec<Tool<T>>,
}
impl<T> BreakTimes<T> {
    pub fn capture<B>(hotbar: &[T]) -> Self
    where
        T: BreakStack<B>,
    {
        // Kotlin filterNot completes before any copy/enchantment reads.
        let selected: Vec<_> = hotbar.iter().filter(|s| !s.is_empty()).collect();
        let mut tools: Vec<_> = selected
            .into_iter()
            .map(|stack| Tool {
                stack: stack.copy_stack(),
                efficiency: stack.efficiency_level(),
            })
            .collect();
        tools.push(Tool {
            stack: T::empty_hand(),
            efficiency: 0,
        });
        Self { tools }
    }
    pub fn ticks<B: BreakState>(&self, state: &B) -> f64
    where
        T: BreakStack<B>,
    {
        let hardness = state.empty_level_hardness();
        if hardness < 0.0 {
            return f64::INFINITY;
        }
        if hardness == 0.0 {
            return 1.0;
        }
        let mut best = f64::INFINITY;
        for tool in &self.tools {
            let harvests = !state.requires_correct_tool() || tool.stack.correct_for_drops(state);
            let divisor = if harvests { 30.0 } else { 100.0 };
            let mut speed = tool.stack.destroy_speed(state);
            if speed > 1.0 && tool.efficiency != 0 {
                speed += (tool.efficiency.wrapping_mul(tool.efficiency) as f32 + 1.0)
                    .clamp(0.0, 1024.0);
            }
            let progress = f64::from(speed) / f64::from(hardness) / divisor;
            let ticks = if progress > 0.0 {
                (1.0 / progress).ceil()
            } else {
                f64::INFINITY
            };
            best = crate::navigation::geometry::java_min(best, ticks);
        }
        best
    }
}
