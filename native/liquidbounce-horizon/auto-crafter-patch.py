"""Actual recipe book entries/requirements, menu engine capacity and ordinary game-mode actions."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/recipe_book.rs',lambda s:s+'\n'+(HERE/'auto-crafter-game.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',(HERE/'auto-crafter-host.rs').read_text(encoding='utf8')+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
 return {'hook_names':[],'helper_names':['client_auto_crafter_snapshot','client_auto_crafter_place','client_auto_crafter_result']}
