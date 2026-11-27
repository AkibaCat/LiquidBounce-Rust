"""Complete first-person ItemChams; apply after ItemESP, no shared ShaderLayer edits."""
from pathlib import Path
HERE=Path(__file__).resolve().parent

def apply(edit,once):
 edit('crates/horizon-client/src/game/item_chams.rs',lambda _:(HERE/'item-chams-game.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client-render/src/gfx/item_chams.rs',lambda _:(HERE/'item-chams-render.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client-render/src/gfx/item-chams.wgsl',lambda _:(HERE/'item-chams.wgsl').read_text(encoding='utf8'))
 for p in (HERE/'item-chams-shaders').glob('*.frag'):
  edit('crates/horizon-client-render/src/gfx/item-chams-shaders/'+p.name,lambda _,p=p:p.read_text(encoding='utf8'))
 def game(t):
  t=once(t,'mod item_esp;','mod item_esp;\nmod item_chams;')
  a='        horizon_client::gfx::outline_glow::begin_frame();'
  return once(t,a,'        self.client_item_chams_frame();\n'+a)
 edit('crates/horizon-client/src/game.rs',game)
 def hooks(t):
  a='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
  t=once(t,a,(HERE/'item-chams-host.rs').read_text(encoding='utf8')+'\n'+a)
  a='        self.client_tracers_reset();'
  return once(t,a,a+'\n        self.client_item_chams_reset();')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 def hand(t):
  a='''            self.submit_item_world(
                &state,
                m,
                block_entity_render::vertex_light(light),
                &mut out,
            );'''
  return once(t,a,'            let _first_person=horizon_client::gfx::item_chams::first_person();\n'+a)
 edit('crates/horizon-client/src/game/frame_geometry.rs',hand)
 def special(t):
  a='            draw(p, "root", pose, adjust, r, color, foil, sink);'
  b='''            if key=="shield" && matches!(texture,"entity/shield/shield_base"|"entity/shield/shield_base_nopattern") {
                let _base=crate::gfx::item_chams::shield_base();
                if crate::gfx::item_chams::shield_active(){
                    let color=crate::gfx::item_chams::shield_tint(color);
                    draw(p,"root",pose,adjust,r,color,foil,&mut ItemChamsShield(sink));
                }else{draw(p,"root",pose,adjust,r,color,foil,sink);}
            }else{draw(p,"root",pose,adjust,r,color,foil,sink);}'''
  t=once(t,a,b)
  return t+'''\n// Only the first-person shield base uses the translucent-cull render type.
struct ItemChamsShield<'a>(&'a mut dyn ItemSink);
impl ItemSink for ItemChamsShield<'_>{fn quad(&mut self,mut q:ItemQuad){if q.color>>24<255{q.translucent=true;q.cull_back=true;}self.0.quad(q);}}
'''
 edit('crates/horizon-client/src/item_render/special.rs',special)
 def item(t):
  a='        // Only plain and foil quads: the translucent sheet\'s overlays are their own material.'
  return once(t,a,'''        // ItemChams base-only mark: texture/banner layers outside this RAII scope
        // retain the original material and tint. Low values 46..49 are reserved here.
        let overlay=if crate::gfx::item_chams::shield_active(){match(glint.is_some(),q.translucent){(false,false)=>46,(true,false)=>47,(false,true)=>48,(true,true)=>49}}else{overlay};
'''+a)
 edit('crates/horizon-client/src/item_render.rs',item)
 def shader(t):
  start=t.index('@vertex fn vs_entity(');end=t.index('\n}',start)+2
  capture=t[start:end].replace('fn vs_entity(','fn vs_item_chams_capture(').replace('    return out;','    out.clip.y=-out.clip.y;\n    return out;')
  t=t[:end]+'\n// Private capture uses GL lower-left texture coordinates.\n'+capture+t[end:]
  a='    if color.a < 0.1 || (in.cull != 0u && !front) { discard; }'
  t=once(t,a,'    let shield_translucent=in.overlay==48u || in.overlay==49u;\n    if select(color.a<0.1,color.a<=0.0,shield_translucent) || (in.cull!=0u && !front){discard;}')
  a='    if in.overlay == 7u || in.overlay == 8u || in.overlay == 33u {'
  t=once(t,a,'    if in.overlay==47u || in.overlay==49u {\n        color=glint_over(color,textureSample(glint_tex,glint_sampler,in.glint_uv)*tint);\n    } else '+a.strip())
  return t
 edit('crates/horizon-client-render/src/entity.wgsl',shader)
 def renderer(t):
  t=once(t,'pub mod item_esp;','pub mod item_esp;\npub mod item_chams;')
  t=once(t,'    hand_group: wgpu::BindGroup,','    hand_group: wgpu::BindGroup,\n    item_chams_hand_group:wgpu::BindGroup,\n    item_chams:item_chams::ItemChamsRenderer,')
  start=t.index('        let hand_group = device.create_bind_group(');end=t.index('\n        });',start)+len('\n        });')
  group=t[start:end].replace('let hand_group =','let item_chams_hand_group =').replace('First-person hand resources','ItemChams first-person hand resources').replace('TextureView(&lightmap_view)','TextureView(item_chams.lightmap_view())')
  a='        let static_entity_layout = device.create_pipeline_layout('
  t=once(t,a,'        let item_chams=item_chams::ItemChamsRenderer::new(&device,overlay_format,false,&entity_shader,&entity_layout);\n'+group+'\n'+a)
  t=once(t,'            hand_group,','            hand_group,\n            item_chams_hand_group,\n            item_chams,')
  assert t.count('pass.set_bind_group(0, &self.hand_group, &[]);')==3
  t=t.replace('pass.set_bind_group(0, &self.hand_group, &[]);','pass.set_bind_group(0, if self.item_chams.lightmap_active(){&self.item_chams_hand_group}else{&self.hand_group}, &[]);')
  a='        let views = self.views(frame);'
  t=once(t,a,'        self.item_chams.prepare(&self.device,&self.queue,&mut encoder,&self.lightmap_view,[self.config.width,self.config.height],frame.title);\n'+a)
  t=once(t,'        let hand_in_world = ','        let hand_in_world = !self.item_chams.pending() && ')
  # Capture the exact same native hand index range. Shader packs can draw this
  # prepared frame earlier; their normal pass is retained and still receives the
  # base tint and the scoped hand lightmap override below.
  a='        if overlay.hand_quads > 0 {'
  t=once(t,a,'        let hand_group=if self.item_chams.lightmap_active(){&self.item_chams_hand_group}else{&self.hand_group};\n        self.item_chams.capture(&self.device,encoder,hand_group,&self.indices,&self.entity_buffer,if frame.title{0}else{(frame.hand.len()/4)as u32},overlay.first_hand_vertex,overlay.real);\n'+a)
  a='        // A native mod\'s turn after the hand, on the real frame only.'
  return once(t,a,'        self.item_chams.composite(&self.device,encoder,view_target,overlay.real);\n'+a)
 edit('crates/horizon-client-render/src/gfx.rs',renderer)
 def pack(t):
  for name in ['draw_hand','draw_hand_water']:
   start=t.index('    fn '+name+'(');pos=t.index('    ) {',start)+len('    ) {')
   t=t[:pos]+'\n        let _item_chams_hand=super::item_chams::pack_hand();'+t[pos:]
  a='        pack_images::copy(encoder, &self.images, pass.image_copies);'
  t=once(t,a,'        let item_chams_lightmap=super::item_chams::pack_lightmap();\n'+a)
  return once(t,'                    Source::Lightmap => &self.lightmap,','                    Source::Lightmap => item_chams_lightmap.as_ref().unwrap_or(&self.lightmap),')
 edit('crates/horizon-client-render/src/gfx/pack_post.rs',pack)
 def pack_hand(t):
  return once(t,'        super::TRANSLUCENT_ITEM_OVERLAY | super::TRANSLUCENT_GLINT_ITEM_OVERLAY','        super::TRANSLUCENT_ITEM_OVERLAY | super::TRANSLUCENT_GLINT_ITEM_OVERLAY | 48 | 49')
 edit('crates/horizon-client-render/src/gfx/pack_hand.rs',pack_hand)
 return {'hook_names':['client_item_chams'],'helper_names':[]}
