from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/auto_deposit.rs',lambda _:(HERE/'auto-deposit-native.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod chest_cleaner_titles;\n','mod chest_cleaner_titles;\nmod auto_deposit;\n'))
 anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,anchor,(HERE/'auto-deposit-host.rs').read_text(encoding='utf8')+'\n'+anchor))
 return {'hook_names':[],'helper_names':['client_auto_deposit_context','client_auto_deposit_target','client_auto_deposit_punch']}
