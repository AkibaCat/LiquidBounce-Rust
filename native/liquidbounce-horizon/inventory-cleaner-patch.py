from pathlib import Path
HERE=Path(__file__).resolve().parent
HELPER_NAMES=['client_inventory_cleaner_context']
def apply(edit,once):
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',(HERE/'inventory-cleaner-host.rs').read_text(encoding='utf8')+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
 return {'hook_names':[],'helper_names':HELPER_NAMES}
