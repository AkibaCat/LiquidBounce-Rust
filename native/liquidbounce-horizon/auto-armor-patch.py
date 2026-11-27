"""AutoArmor's real player attributes and vanilla inventory screen effects."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,anchor,(HERE/'auto-armor-host.rs').read_text(encoding='utf8')+'\n'+anchor))
 return {'hook_names':[],'helper_names':['client_auto_armor_context','client_auto_armor_screen']}
