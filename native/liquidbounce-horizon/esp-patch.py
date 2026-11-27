"""Full ESP plus shared OutlineGlow and existing StorageESP entity bridge."""
from pathlib import Path
import runpy
HERE=Path(__file__).resolve().parent
NATIVE=HERE if(HERE/'storage-esp-render.py').is_file()else HERE.parents[1]

def apply(edit,once):
 edit('crates/horizon-client/src/game/esp.rs',lambda _:(HERE/'esp-game.rs').read_text(encoding='utf8'))
 render=runpy.run_path(str(NATIVE/'storage-esp-render.py'))
 edit('crates/horizon-client-render/src/gfx/esp_plain.rs',lambda _:render['plain_source']().replace('StoragePlainRenderer','EspPlainRenderer').replace('ModuleStorageESP','ModuleESP'))
 edit('crates/horizon-client-render/src/gfx/outline_owner.rs',lambda _:runpy.run_path(str(HERE/'outline-glow-render.py'))['source']())
 edit('crates/horizon-client-render/src/gfx/outline_glow.rs',lambda _:(HERE/'outline-glow-render.rs').read_text(encoding='utf8'))
 for name in ['outline-glow.wgsl','outline-glow-blur-h.frag','outline-glow-blur-v.frag']:
  edit('crates/horizon-client-render/src/gfx/'+name,lambda _,name=name:(HERE/name).read_text(encoding='utf8'))
 def game(t):
  t=once(t,'mod tracers;','mod tracers;\nmod esp;')
  a='        if !title {self.client_storage_esp_frame(camera,alpha);}else{'
  return once(t,a,'        horizon_client::gfx::outline_glow::begin_frame();\n        if !title {self.client_esp_frame(camera,alpha);}else{self.client_esp_reset();}\n'+a)
 edit('crates/horizon-client/src/game.rs',game)
 def hooks(t):
  a='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
  t=once(t,a,(HERE/'esp-host.rs').read_text(encoding='utf8')+'\n'+a)
  t=once(t,'        self.client_tracers_tick();','        self.client_tracers_tick();\n        self.client_esp_tick();')
  return once(t,'        self.client_tracers_reset();','        self.client_tracers_reset();\n        self.client_esp_reset();')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 def tracers_visibility(t):
  if '    pub(super) fn client_tracers_entities(' in t:return t
  return once(t,'    fn client_tracers_entities(','    pub(super) fn client_tracers_entities(')
 edit('crates/horizon-client/src/game/tracers.rs',tracers_visibility)
 def overlay(t):
  a='            self.client_target_renderer_overlay(&mut batch);'
  return once(t,a,a+'\n            self.client_esp_overlay(&mut batch);')
 # ESP 2D is drawn at the same HUD projection point as the original world markers.
 edit('crates/horizon-client/src/game.rs',overlay)
 def storage(t):
  a='        horizon_client::gfx::storage_esp::begin_frame(&v["body"].to_string());'
  return once(t,a,a+'\n        let mut entities=v["body"].clone();entities["quads"]=json!([]);entities["lines"]=json!([]);\n        horizon_client::gfx::outline_glow::submit(horizon_client::gfx::outline_glow::Owner::StorageEsp,&entities.to_string());')
 edit('crates/horizon-client/src/game/storage_esp.rs',storage)
 def storage_renderer(t):
  t=once(t,'pub fn capture_entity(id:i32,vertices:&[crate::mesher::Vertex]){if let Some(color)=entity_color(id){capture_model(vertices,color);}}','pub fn capture_entity(id:i32,vertices:&[crate::mesher::Vertex]){super::outline_glow::capture_entity(id,vertices);}')
  t=once(t,'const SOURCES: [&str; 12]','const SOURCES: [&str; 14]')
  t=once(t,'pipelines: [wgpu::RenderPipeline; 12]','pipelines: [wgpu::RenderPipeline; 14]')
  t=once(t,'    include_str!("storage-esp-sobel.frag"),','    include_str!("storage-esp-sobel.frag"),\n    include_str!("outline-glow-blur-h.frag"),\n    include_str!("outline-glow-blur-v.frag"),')
  return once(t,'if effect.style==9{let group=group(&targets.views[0]);pass(encoder,&targets.views[3],&self.pipelines[11],Some(&group),false,true);}', 'if effect.style==9{for(source,dest,pipeline)in[(0,1,11),(1,2,12),(2,3,13)]{let group=group(&targets.views[source]);pass(encoder,&targets.views[dest],&self.pipelines[pipeline],Some(&group),false,true);}}')
 edit('crates/horizon-client-render/src/gfx/storage_esp.rs',storage_renderer)
 def glowing(t):
  a='        outline.map(|rgb| rgb & 0xFF_FFFF)'
  t=once(t,a,'        if outline.is_some(){horizon_client::gfx::outline_glow::promote_entity(id as i32);}\n        if horizon_client::gfx::outline_glow::entity_owner(id as i32).is_some(){None}else{'+a.strip()+'}')
  a='        let color = horizon_client::mobs::render::abgr(rgb);'
  return once(t,a,'        if horizon_client::gfx::outline_glow::active(){horizon_client::gfx::outline_glow::capture_untagged(vertices,0xff000000|(rgb&0xffffff));return;}\n'+a)
 edit('crates/horizon-client/src/game/glowing.rs',glowing)
 def skin(t):
  a='!horizon_client::gfx::storage_esp::active()'
  assert t.count(a)==2
  return t.replace(a,a+' && !horizon_client::gfx::outline_glow::active()')
 edit('crates/horizon-client/src/game/creatures.rs',skin)
 def mobs(t):
  a='                && !horizon_client::visual_environment::allows(horizon_client::visual_environment::Choice::InvisibleEntities) { continue; }'
  t=once(t,a,'                && !horizon_client::gfx::outline_glow::active()\n'+a)
  a='            let storage=horizon_client::gfx::storage_esp::entity_color(mob.id as i32).is_some();'
  t=once(t,a,'')
  a='            let outline = self.mob_outline(mob);'
  t=once(t,a,a+'\n            let storage=horizon_client::gfx::storage_esp::entity_color(mob.id as i32).is_some() || horizon_client::gfx::outline_glow::entity_owner(mob.id as i32).is_some();\n            if horizon_client::mobs::trader::is_invisible(mob) && true_sight.is_none() && !storage && !horizon_client::visual_environment::allows(horizon_client::visual_environment::Choice::InvisibleEntities){continue;}')
  a='let layers_only = mob.invisible && true_sight.is_none() && outline.is_none() && is_player;'
  return once(t,a,a.replace('&& is_player','&& !storage && is_player'))
 edit('crates/horizon-client/src/game/creatures/mob_geometry.rs',mobs)
 def model(t):
  a='crate::gfx::storage_esp::entity_color(m.id as i32).is_none()'
  assert t.count(a)==2
  t=t.replace(a,a+' && crate::gfx::outline_glow::entity_owner(m.id as i32).is_none()')
  a='&& !crate::visual_environment::allows(crate::visual_environment::Choice::InvisibleEntities) { return; }'
  return once(t,a,'&& !crate::true_sight::active() '+a)
 edit('crates/horizon-client/src/mobs/render.rs',model)
 def vehicle(t,var,method):
  start=t.index('    pub(super) fn '+method+'(');end=t.index('\n    }',start)+6;b=t[start:end]
  a=f'let storage=horizon_client::gfx::storage_esp::entity_color({var}.id as i32).is_some();'
  b=once(b,a,f'let outline=self.thing_outline({var}.id,'+('if b.chest{"minecraft:chest_boat"}else{"minecraft:boat"}'if var=='b'else'c.kind.id()')+');\n            '+a[:-1]+f' || horizon_client::gfx::outline_glow::entity_owner({var}.id as i32).is_some();')
  a=f'            horizon_client::gfx::storage_esp::capture_entity({var}.id as i32,&out[entity_start..]);'
  b=once(b,a,a+'\n            if let Some(rgb)=outline{self.mark_outline(&out[entity_start..],rgb);}')
  return t[:start]+b+t[end:]
 edit('crates/horizon-client/src/game/boats.rs',lambda t:vehicle(t,'b','boat_geometry'))
 edit('crates/horizon-client/src/game/minecarts.rs',lambda t:vehicle(t,'c','cart_geometry'))
 def armor(t):
  a='            let chams=self.client_chams_entity(a.id as i32);'
  t=once(t,a,a+'\n            let outline=self.thing_outline(a.id,"minecraft:armor_stand");\n            let shared=horizon_client::gfx::outline_glow::entity_owner(a.id as i32).is_some();')
  t=once(t,'            if !chams && culling.hides_entity() {','            if !chams && !shared && outline.is_none() && culling.hides_entity() {')
  a='            if chams {horizon_client::gfx::chams::mark_entity(&mut out[entity_start..]);}'
  t=once(t,a,'            if shared && a.invisible && true_sight.is_none(){let mut visible=a.clone();visible.invisible=false;let mut silhouette=Vec::new();self.armor_stand_vertices(&visible,base,packed,None,&mut silhouette);horizon_client::gfx::outline_glow::capture_entity(a.id as i32,&silhouette);}else{horizon_client::gfx::outline_glow::capture_entity(a.id as i32,&out[entity_start..]);}\n            if let Some(rgb)=outline{self.mark_outline(&out[entity_start..],rgb);}\n'+a)
  t=once(t,'            if !chams && !self.entity_glowing(a.id) {','            if !chams && !shared && outline.is_none() && !self.entity_glowing(a.id) {')
  a='        if a.invisible && !horizon_client::visual_environment::allows(horizon_client::visual_environment::Choice::InvisibleEntities) { return; }'
  t=once(t,a,a.replace('a.invisible &&','a.invisible && !horizon_client::true_sight::active() && horizon_client::gfx::outline_glow::entity_owner(a.id as i32).is_none() &&'))
  return t
 edit('crates/horizon-client/src/game/armor_stands.rs',armor)
 def player(t):
  start=t.index('    pub(super) fn player_model_geometry(');end=t.index('\n    /// `extractEntityInInventory',start);b=t[start:end]
  a='        if self.own_invisible() && !horizon_client::visual_environment::allows(horizon_client::visual_environment::Choice::InvisibleEntities) { return; }'
  early='''        let vanilla=self.potions.effects.get("glowing").map(|_|super::glowing::team_color(self.own_team()));
        let outline=self.entity_outline(u32::try_from(self.own_entity_id()).unwrap_or(0),"minecraft:player",vanilla);
        let shared=horizon_client::gfx::outline_glow::entity_owner(self.own_entity_id()).is_some();
'''
  b=once(b,a,early+a.replace('self.own_invisible() &&','self.own_invisible() && !shared && self.client_true_sight_colors(self.own_entity_id()).is_none() &&'))
  a='            if chams {horizon_client::gfx::chams::mark_entity(&mut out[start..]);}'
  b=once(b,a,'            horizon_client::gfx::outline_glow::capture_entity(self.own_entity_id(),&out[start..]);\n'+a)
  a='''        let vanilla = self
            .potions
            .effects
            .get("glowing")
            .map(|_| super::glowing::team_color(self.own_team()));
        let id = u32::try_from(self.own_entity_id()).unwrap_or(0);
        if let Some(color) = self.entity_outline(id, "minecraft:player", vanilla) {'''
  b=once(b,a,'        if shared && !body{let mut silhouette=Vec::new();horizon_client::mobs::render::render(m,alpha as f32,camera,&lookup,packed,self.res.mob_data(),&mut silhouette);horizon_client::gfx::outline_glow::capture_entity(self.own_entity_id(),&silhouette);}\n        horizon_client::gfx::outline_glow::capture_entity(self.own_entity_id(),&out[start..]);\n        if let Some(color) = outline {')
  return t[:start]+b+t[end:]
 edit('crates/horizon-client/src/game/perspective.rs',player)
 def crystal(t):
  start=t.index('    fn end_crystal_geometry(');end=t.index('\n    /// EnderEyeItem',start);b=t[start:end]
  a='            let start = out.len();';b=once(b,a,'            let outline=self.thing_outline(crystal.id,"minecraft:end_crystal");\n'+a)
  a='            if let Some(target) = crystal.beam {'
  b=once(b,a,'            horizon_client::gfx::outline_glow::capture_entity(crystal.id as i32,&out[start..]);\n            if let Some(rgb)=outline{self.mark_outline(&out[start..],rgb);}\n'+a)
  return t[:start]+b+t[end:]
 edit('crates/horizon-client/src/game/end.rs',crystal)
 def dragon(t):
  a='        let first = out.len();\n        horizon_client::mobs::render::draw(model, base, &poses, skin, color, light, overlay, out);'
  t=once(t,a,'        let outline=self.entity_outline(d.id,"minecraft:ender_dragon",self.entity_glowing(d.id).then_some(super::glowing::team_color(self.mob_team(d.id))));\n'+a)
  a='        // RenderTypes.eyes: the glowing eyes, added on top.'
  t=once(t,a,'        horizon_client::gfx::outline_glow::capture_entity(d.id as i32,&out[first..]);\n        if let Some(rgb)=outline{self.mark_outline(&out[first..],rgb);}\n'+a)
  start=t.index('    pub(super) fn dragon_fireball_geometry(');end=t.index('\n    pub(super) fn fight_ready',start);b=t[start:end]
  b=once(b,'            // DragonFireballRenderer: scale 2, full bright.','            let capture_start=out.len();\n            let outline=self.thing_outline(b.id,"minecraft:dragon_fireball");\n            // DragonFireballRenderer: scale 2, full bright.')
  a='''            }
        }
    }'''
  b=once(b,a,'            }\n            horizon_client::gfx::outline_glow::capture_entity(b.id as i32,&out[capture_start..]);\n            if let Some(rgb)=outline{self.mark_outline(&out[capture_start..],rgb);}\n        }\n    }')
  return t[:start]+b+t[end:]
 edit('crates/horizon-client/src/game/dragon.rs',dragon)
 def item_projectile(t,var,method,kind):
  start=t.index('    pub(super) fn '+method+'(');end=t.index('\n    }',start)+6;b=t[start:end]
  a='            self.submit_item_world(&state, pose, vertex_light(light), out);'
  b=once(b,a,f'            let outline=self.thing_outline({var}.id,{kind});\n            let capture_start=out.len();\n'+a+f'\n            horizon_client::gfx::outline_glow::capture_entity({var}.id as i32,&out[capture_start..]);\n            if let Some(rgb)=outline{{self.mark_outline(&out[capture_start..],rgb);}}')
  return t[:start]+b+t[end:]
 def potions(t):
  t=item_projectile(t,'t','thrown_potion_geometry','thrown_kind(&t.stack)')
  return t+'''\nimpl Game<'_>{pub(super) fn client_esp_potions(&self,alpha:f64)->Vec<serde_json::Value>{self.potions.thrown.iter().map(|t|super::esp::object(t.id,(u64::from(t.id)<<8)|0x87,thrown_kind(&t.stack),t.position,t.previous,0.25,0.25,alpha,2,false)).collect()}}\n'''
 edit('crates/horizon-client/src/game/potions.rs',potions)
 edit('crates/horizon-client/src/game/fireworks.rs',lambda t:item_projectile(t,'r','rocket_geometry','"minecraft:firework_rocket"'))
 def fishing(t):
  a='''        let hooks: Vec<glam::DVec3> = self
            .hook_position(alpha)
            .into_iter()
            .chain(self.remote_hook_ends(alpha).into_iter().map(|(h, _)| h))
            .collect();'''
  b='''        let mut hooks=Vec::<(u32,glam::DVec3)>::new();
        if let(Some(h),Some(p))=(&self.hook,self.hook_position(alpha)){hooks.push((h.id,p));}
        for h in &self.remote_hooks{if self.mobs.iter().any(|m|m.id==h.owner){hooks.push((h.id,glam::DVec3::new(h.previous.x+(h.position.x-h.previous.x)*alpha,h.previous.y+(h.position.y-h.previous.y)*alpha,h.previous.z+(h.position.z-h.previous.z)*alpha)));}}
'''
  t=once(t,a,b)
  start=t.index('    pub(super) fn fishing_geometry(');end=t.index('\n    /// FishingHookRenderer: each line',start);b=t[start:end]
  b=once(b,'        for p in hooks {','        for(id,p)in hooks {\n            let capture_start=out.len();\n            let outline=self.thing_outline(id,"minecraft:fishing_bobber");')
  a='''            }
        }
    }'''
  b=once(b,a,'            }\n            horizon_client::gfx::outline_glow::capture_entity(id as i32,&out[capture_start..]);\n            if let Some(rgb)=outline{self.mark_outline(&out[capture_start..],rgb);}\n        }\n    }')
  t=t[:start]+b+t[end:]
  return t+'''\nimpl Game<'_>{pub(super) fn client_esp_hooks(&self,alpha:f64)->Vec<serde_json::Value>{let mut out=vec![];if let Some(h)=&self.hook{out.push(super::esp::object(h.id,(u64::from(h.id)<<8)|0x88,"minecraft:fishing_bobber",h.position,h.previous,0.25,0.25,alpha,2,false));}for h in &self.remote_hooks{out.push(super::esp::object(h.id,(u64::from(h.id)<<8)|0x89,"minecraft:fishing_bobber",h.position,h.previous,0.25,0.25,alpha,2,false));}out}}\n'''
 edit('crates/horizon-client/src/game/fishing.rs',fishing)
 def gfx(t):
  t=once(t,'pub mod tracers;','pub mod tracers;\nmod outline_owner;\npub mod outline_glow;\npub mod esp_plain;')
  t=once(t,'    tracers: tracers::TracerRenderer,','    tracers: tracers::TracerRenderer,\n    outline_glow:outline_glow::OutlineGlowRenderer,\n    esp_plain:esp_plain::EspPlainRenderer,')
  a='        let tracers=tracers::TracerRenderer::new(&device,frame_format,linear_light);'
  t=once(t,a,a+'\n        let outline_glow=outline_glow::OutlineGlowRenderer::new(&device,frame_format,linear_light,&entity_layout);\n        let esp_plain=esp_plain::EspPlainRenderer::new(&device,frame_format,linear_light);')
  t=once(t,'            tracers,','            tracers,\n            outline_glow,esp_plain,')
  a='        self.tracers.prepare(&self.device,jittered,frame.camera,breadcrumbs_size,frame.title);'
  t=once(t,a,a+'\n        self.outline_glow.prepare(&self.device,jittered,frame.camera,breadcrumbs_size,frame.title);\n        self.esp_plain.prepare(&self.device,jittered,frame.camera,breadcrumbs_size,frame.title);')
  t=once(t,'        let hand_in_world = ','        let hand_in_world = !self.outline_glow.pending() && !self.esp_plain.pending() && ')
  a='        self.storage_plain.draw(&self.device,&mut encoder,world_view,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees);'
  return once(t,a,a+'\n        self.outline_glow.draw(&self.device,&mut encoder,world_view,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees,&self.entity_group);\n        self.esp_plain.draw(&self.device,&mut encoder,world_view,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees);')
 edit('crates/horizon-client-render/src/gfx.rs',gfx)
 return {'hook_names':['client_esp'],'helper_names':[]}
