from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 a='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,a,(HERE/'packet-logger-host.rs').read_text(encoding='utf8')+'\n'+a))
 return {'hook_names':[],'helper_names':['client_packet_logger_chat']}
