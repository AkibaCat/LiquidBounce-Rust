from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/auto_wind_charge.rs',lambda _:(HERE/'auto-wind-charge-native.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod auto_walk_runtime;','mod auto_walk_runtime;\nmod auto_wind_charge;'))
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',(HERE/'auto-wind-charge-host.rs').read_text(encoding='utf8')+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
 return {'hook_names':[],'helper_names':['client_auto_wind_charge_snapshot','client_auto_wind_charge_query']}
