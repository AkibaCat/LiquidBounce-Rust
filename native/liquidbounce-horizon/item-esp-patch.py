"""ItemESP model captures and its independent plain renderer, after shared ESP."""
from pathlib import Path
import runpy
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/item_esp.rs',lambda _:(HERE/'item-esp-game.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client-render/src/gfx/item_esp.rs',lambda _:runpy.run_path(str(HERE/'item-esp-render.py'))['source']())
 def game(t):
  t=once(t,'mod tracers;','mod tracers;\nmod item_esp;')
  return once(t,'        horizon_client::gfx::outline_glow::begin_frame();','        horizon_client::gfx::outline_glow::begin_frame();\n        self.client_item_esp_frame(alpha,camera);')
 edit('crates/horizon-client/src/game.rs',game)
 def hooks(t):
  a='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
  t=once(t,a,(HERE/'item-esp-host.rs').read_text(encoding='utf8')+'\n'+a)
  t=once(t,'        self.client_tracers_tick();','        self.client_tracers_tick();\n        self.client_item_esp_tick();')
  return once(t,'        self.client_tracers_reset();','        self.client_tracers_reset();\n        super::item_esp::clear();')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 def item(t):
  t=once(t,'            let entity_start = out.len();','            let entity_start = out.len();\n            let item_outline=if entity {self.thing_outline(item.id,"minecraft:item")}else{None};\n            let shared_outline=entity && horizon_client::gfx::outline_glow::entity_owner(item.id as i32).is_some();')
  t=once(t,'            let culling = entity.then(|| {','            let culling = (entity && !shared_outline).then(|| {')
  t=once(t,'            let outline = self.thing_outline(item.id, "minecraft:item");','            let outline = if shared_outline {None}else{item_outline};')
  t=once(t,'            let skinned = skinning\n','            let skinned = (skinning && !shared_outline)\n')
  a='            if let Some(rgb) = outline {'
  return once(t,a,'            if shared_outline {\n                horizon_client::gfx::outline_glow::capture_entity(item.id as i32,&out[entity_start..]);\n            } else '+a.strip() )
 edit('crates/horizon-client/src/game/item_draw.rs',item)
 def projectiles(t):
  # Delayed boundary handles ALL continue branches without capturing pickup particles.
  a='''            (a, at)
        });
        // ItemPickupParticle: a picked-up arrow flying to its collector.'''
  t=once(t,a,a.replace('(a, at)','(a, at, true)'))
  a='Some((&**a, super::super::pickups::position(p, alpha)))'
  t=once(t,a,'Some((&**a, super::super::pickups::position(p, alpha),false))')
  a='''        let mut glowing = None;
        for (a, at) in live.chain(taken) {'''
  t=once(t,a,'''        let mut glowing = None;
        let mut shared_capture:Option<(i32,usize)>=None;
        for (a, at, live_entity) in live.chain(taken) {
            if let Some((id,start))=shared_capture.take(){horizon_client::gfx::outline_glow::capture_entity(id,&out[start..]);}
            let projectile_outline=if a.id!=0 {self.thing_outline(a.id,a.kind.id())}else{None};
            let shared_outline=live_entity && horizon_client::gfx::outline_glow::entity_owner(a.id as i32).is_some();
            if shared_outline {shared_capture=Some((a.id as i32,out.len()));}''')
  t=once(t,'            if a.id != 0\n                && let Some(rgb) = self.thing_outline(a.id, a.kind.id())','            if !shared_outline && let Some(rgb) = projectile_outline')
  return once(t,'        if let Some((start, rgb)) = glowing {','        if let Some((id,start))=shared_capture {horizon_client::gfx::outline_glow::capture_entity(id,&out[start..]);}\n        if let Some((start, rgb)) = glowing {')
 edit('crates/horizon-client/src/game/creatures/mob_geometry.rs',projectiles)
 def trident(t):
  t=once(t,'                (t, at)\n            });','                (t, at, true)\n            });')
  t=once(t,'Some((&**t, super::pickups::position(p, alpha)))','Some((&**t, super::pickups::position(p, alpha),false))')
  t=once(t,'        for (t, at) in live.chain(taken) {','        for (t, at, live_entity) in live.chain(taken) {\n            let capture_start=out.len();')
  a='''        }
        self.mark_motion(out.len(), 0);
    }
}
horizon_client::impl_joined!(ThrownTrident);'''
  return once(t,a,'''            if live_entity {horizon_client::gfx::outline_glow::capture_entity(t.id as i32,&out[capture_start..]);}
'''+a)
 edit('crates/horizon-client/src/game/trident.rs',trident)
 def renderer(t):
  t=once(t,'pub mod tracers;','pub mod tracers;\npub mod item_esp;')
  t=once(t,'    tracers: tracers::TracerRenderer,','    tracers: tracers::TracerRenderer,\n    item_esp: item_esp::ItemEspRenderer,')
  a='        let tracers=tracers::TracerRenderer::new(&device,frame_format,linear_light);'
  t=once(t,a,a+'\n        let item_esp=item_esp::ItemEspRenderer::new(&device,frame_format,linear_light);')
  t=once(t,'            tracers,','            tracers,\n            item_esp,')
  a='        self.tracers.prepare(&self.device,jittered,frame.camera,breadcrumbs_size,frame.title);'
  t=once(t,a,a+'\n        self.item_esp.prepare(&self.device,jittered,frame.camera,breadcrumbs_size,frame.title);')
  a='        let hand_in_world = '
  t=once(t,a,a+'!self.item_esp.pending() && ')
  a='        self.tracers.draw(&self.device,&mut encoder,world_view,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees);'
  return once(t,a,a+'\n        self.item_esp.draw(&self.device,&mut encoder,world_view,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees);')
 edit('crates/horizon-client-render/src/gfx.rs',renderer)
 return {'hook_names':['client_item_esp'],'helper_names':[]}
