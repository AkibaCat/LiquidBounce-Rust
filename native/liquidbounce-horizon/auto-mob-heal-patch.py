"""Actual AutoMobHeal mob/component snapshots and targeted interaction."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/auto_mob_heal.rs',lambda _:(HERE/'auto-mob-heal-native.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod auto_rod;','mod auto_rod;\nmod auto_mob_heal;'))
 anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,anchor,(HERE/'auto-mob-heal-host.rs').read_text(encoding='utf8')+'\n'+anchor))
 return {'hook_names':[],'helper_names':['client_auto_mob_heal_query']}
