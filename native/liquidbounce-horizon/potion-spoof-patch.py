from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda t:once(t,anchor,(HERE/'potion-spoof-host.rs').read_text(encoding='utf8')+'\n'+anchor))
 return {'hook_names':[],'helper_names':['client_potion_spoof_duration','client_potion_spoof_apply']}
