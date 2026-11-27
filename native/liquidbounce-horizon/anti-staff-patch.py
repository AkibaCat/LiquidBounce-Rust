from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/anti_staff.rs',lambda _:(HERE/'anti-staff-network.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/client_lib.rs',lambda t:once(t,'pub mod freeze_bridge;','pub mod freeze_bridge;\npub mod anti_staff;'))
 a='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda t:once(t,a,(HERE/'anti-staff-host.rs').read_text(encoding='utf8')+'\n'+a))
 def render(t):
  t=once(t,'    error: bool,','    error: bool,\n    staffs: Vec<String>,\n    staff_suffix:String,')
  t=once(t,'            error: !v["error"].is_null(),','            error: !v["error"].is_null(),\n            staffs:v["staffs"].as_array().into_iter().flatten().filter_map(|v|v.as_str().map(str::to_owned)).collect(),\n            staff_suffix:v["staffSuffix"].as_str().unwrap_or(" - (Staff)").into(),')
  t=once(t,'        return Text::literal(&player.name);','        original=Text::literal(&player.name);')
  t=once(t,'    if p.show_game_mode {','    if p.show_game_mode && p.visibility & 4 == 0 {')
  return once(t,'    original\n}\npub fn visible','    if p.staffs.iter().any(|s|s==&player.name){original.push(Text::styled(p.staff_suffix,crate::text::Style::color(0xffb4b4)));}\n    original\n}\npub fn visible')
 edit('crates/horizon-client-render/src/better_tab.rs',render)
 return {'hook_names':[],'helper_names':['client_anti_staff_chat']}
