"""Expose the cached native monitor refresh rate for real browser/game FPS settings."""
def apply(edit,once):
 edit('crates/horizon-client/src/game/debug_keys.rs',lambda s:once(s,'    fn debug_refresh_rate(&self) -> Option<u32> {','    pub(super) fn debug_refresh_rate(&self) -> Option<u32> {'))
 anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 helper='    pub(super) fn native_client_click_gui_refresh_rate(&self)->u32 {self.debug_refresh_rate().map(|mhz|(mhz/1000).max(1)).unwrap_or(60)}\n\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,anchor,helper+anchor))
 return {'hook_names':[],'helper_names':['client_click_gui_refresh_rate']}
