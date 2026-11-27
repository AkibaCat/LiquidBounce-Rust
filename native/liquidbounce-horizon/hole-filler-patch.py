"""Shared HoleManager subscriptions for HoleESP and HoleFiller, plus native snapshots."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/hole_filler.rs',lambda _:(HERE/'hole-filler-native.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod hole_esp;','mod hole_esp;\nmod hole_filler;'))
 anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,anchor,(HERE/'hole-filler-host.rs').read_text(encoding='utf8')+'\n'+anchor))
 def manager(s):
  s=once(s,'    layout: Option<(i32, i32)>,','    layout: Option<(i32, i32)>,\n    subscribers: BTreeSet<u64>,')
  s=once(s,'    /// Real LocalPlayer post-tick: manager movement and ChunkScanner-equivalent diffs.','''    fn client_hole_subscriptions(&self)->Vec<(u64,i32,i32)>{
        let mut subscriptions=vec![];
        if let Some((h,v))=self.client_hole_esp_options(){subscriptions.push((0,h,v));}
        let mut result=String::new();self.native_client_hole_filler("{\\"action\\":\\"options\\"}",&mut result);
        if let Ok(v)=serde_json::from_str::<Value>(&result){if v["active"]==true{if let Some(range)=v["range"].as_i64(){subscriptions.push((1,range as i32,range as i32));}}}
        subscriptions
    }
    /// Real LocalPlayer post-tick: manager movement and ChunkScanner-equivalent diffs.''')
  old='''        let (Some((h, v)), Some(player)) = (self.client_hole_esp_options(), self.player.as_ref())
        else {'''
  new='''        let subscriptions=self.client_hole_subscriptions();
        let Some(player)=self.player.as_ref().filter(|_|!subscriptions.is_empty()) else {'''
  s=once(s,old,new)
  old='''        if !cache.manager.active() {
            cache.manager.subscribe(0, h, v, position, max_y, &mut read);
        } else {
            cache.manager.settings(0, h, v);
            cache.manager.player_post_tick(position, max_y, &mut read);
        }'''
  new='''        let next=subscriptions.iter().map(|(id,_,_)|*id).collect::<BTreeSet<_>>();
        // Subscribe new owners before removing old ones, preserving a shared cache.
        for (id,h,v) in subscriptions{if cache.subscribers.contains(&id){cache.manager.settings(id,h,v);}else{cache.manager.subscribe(id,h,v,position,max_y,&mut read);}}
        for id in cache.subscribers.difference(&next).copied().collect::<Vec<_>>(){cache.manager.unsubscribe(id);}
        cache.subscribers=next;
        cache.manager.player_post_tick(position,max_y,&mut read);'''
  s=once(s,old,new)
  return once(s,'''        if self.client_hole_esp_options().is_none() {
            self.client_hole_esp_reset();
            return;
        }''','''        if self.client_hole_esp_options().is_none() {
            horizon_client::gfx::hole_esp::begin_frame("null");
            if self.client_hole_subscriptions().is_empty(){self.client_hole_esp_reset();}
            return;
        }''')
 edit('crates/horizon-client/src/game/hole_esp.rs',manager)
 return {'hook_names':['client_hole_filler'],'helper_names':['client_hole_filler_query']}
