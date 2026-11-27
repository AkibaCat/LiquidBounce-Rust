"""Read actual resident chunk snapshots and render original ProtectionZones geometry."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/protection_zones.rs',lambda _:(HERE/'protection-zones-game.rs').read_text(encoding='utf8'))
 def game(t):
  t=once(t,'mod native_visuals;','mod native_visuals;\nmod protection_zones;')
  t=once(t,'    kept_block_entities: std::cell::RefCell<native_block_entity_draw::KeptGeometry>,','    client_protection_zones_cache: std::cell::RefCell<protection_zones::Cache>,\n    kept_block_entities: std::cell::RefCell<native_block_entity_draw::KeptGeometry>,')
  return once(t,'            kept_block_entities: Default::default(),','            client_protection_zones_cache: Default::default(),\n            kept_block_entities: Default::default(),')
 edit('crates/horizon-client/src/game.rs',game)
 anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda t:once(t,anchor,(HERE/'protection-zones-host.rs').read_text(encoding='utf8')+'\n'+anchor))
 edit('crates/horizon-client/src/game/world_primitives.rs',lambda t:once(t,'        self.client_block_outline_geometry(&mut primitives);','        self.client_block_outline_geometry(&mut primitives);\n        self.client_protection_zones_geometry(&mut primitives);'))
 return {'hook_names':['client_protection_zones'],'helper_names':[]}
