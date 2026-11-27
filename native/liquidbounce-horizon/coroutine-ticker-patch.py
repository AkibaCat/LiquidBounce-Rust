#!/usr/bin/env python3
"""Retain Minecraft.tick depth over all native tick handlers and Host reentry."""
SCOPE = '''
thread_local! {
    static COROUTINE_TICK_DEPTH: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
pub(super) struct CoroutineTickScope;
impl CoroutineTickScope {
    pub(super) fn enter() -> Self {
        COROUTINE_TICK_DEPTH.with(|depth| depth.set(depth.get().checked_add(1).expect("Minecraft tick depth exhausted")));
        Self
    }
}
impl Drop for CoroutineTickScope {
    fn drop(&mut self) { COROUTINE_TICK_DEPTH.with(|depth| depth.set(depth.get() - 1)); }
}
'''
HELPER = '''    /// CoroutineTicker resumes only in the outer Minecraft.tick, including
    /// reentry that occurs after its callback batch has already completed.
    pub(super) fn native_client_coroutine_outermost(&self) -> bool {
        COROUTINE_TICK_DEPTH.with(|depth| depth.get() == 1)
    }
'''

def apply(edit, once):
    edit('crates/horizon-client/src/game/native_hooks.rs', lambda text: once(text,
        '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',
        HELPER + '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n') + SCOPE)
    edit('crates/horizon-client/src/game.rs', lambda text: once(text,
        '                    for _ in 0..steps {\n                        let started = Instant::now();',
        '                    for _ in 0..steps {\n                        let _native_tick_scope = native_hooks::CoroutineTickScope::enter();\n                        let started = Instant::now();'))
    return {'hook_names': [], 'helper_names': ['client_coroutine_outermost']}
