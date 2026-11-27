from pathlib import Path
HERE=Path(__file__).resolve().parent

def apply(edit,once):
    edit('crates/horizon-client/src/game/placement_esp.rs',lambda _:(HERE/'placement-esp-game.rs').read_text(encoding='utf8'))
    edit('crates/horizon-client/src/game/placement_esp_search.rs',lambda _:(HERE.parent/'liquidbounce-behaviors/src/void_search.rs').read_text(encoding='utf8'))
    edit('crates/horizon-client/src/game.rs',lambda t:once(t,'mod native_visuals;','mod native_visuals;\nmod placement_esp;\nmod placement_esp_search;'))
    def hooks(t):
        anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
        t=once(t,anchor,(HERE/'placement-esp-host.rs').read_text(encoding='utf8')+'\n'+anchor)
        t=once(t,'    pub(super) fn native_client_tick_pre(&mut self) {','    pub(super) fn native_client_tick_pre(&mut self) {\n        self.client_placement_esp_tick();')
        t=once(t,'        self.native_client_player_tick(&mut cancelled);','        self.native_client_player_tick(&mut cancelled);\n        self.client_placement_esp_player_tick();')
        return once(t,'    pub(super) fn native_client_packet(&mut self, packet: &ClientboundPlayPacket) {','    pub(super) fn native_client_packet(&mut self, packet: &ClientboundPlayPacket) {\n        self.client_placement_esp_packet(packet);')
    edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
    edit('crates/horizon-client/src/game/world_primitives.rs',lambda t:once(t,'        self.client_block_outline_geometry(&mut primitives);','        self.client_block_outline_geometry(&mut primitives);\n        self.client_placement_esp_geometry(&mut primitives);'))
    anchor='    pub fn is_face_sturdy_center(&self, state: BlockStateId, direction: Direction) -> bool {'
    wrapper='    /// Original UP + CENTER block-support query for the LiquidBounce native adapter.\n    pub fn client_placement_esp_sturdy<L: LevelAccess + ?Sized>(&self, level: &mut L, ticks: &mut Ticks, pos: BlockPos) -> bool { self.sim(level, ticks).can_support_center(pos, Direction::Up) }\n'
    edit('crates/horizon-world/src/level/mod.rs',lambda t:once(t,anchor,wrapper+anchor))
    return {'hook_names':['client_placement_esp'],'helper_names':[]}
