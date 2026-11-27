"""Fresh synchronous FakeLag player/target context before original packet dispatch."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/fake_lag.rs',lambda _:(HERE/'fake-lag-native.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod auto_shoot;','mod auto_shoot;\nmod fake_lag;'))
 a='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,a,(HERE/'fake-lag-host.rs').read_text(encoding='utf8')+'\n'+a))
 edit('crates/horizon-client/src/game/remote.rs',lambda s:once(s,'        let mut criticals=self.native_client_criticals_context();','        self.native_client_fake_lag_capture();\n        let mut criticals=self.native_client_criticals_context();'))
 return {'hook_names':['client_fake_lag_capture'],'helper_names':['client_fake_lag_query']}
