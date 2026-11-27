"""PotionFX actual packet/entity snapshots, complete nine textures and GPU consumer."""
from pathlib import Path
import runpy
HERE=Path(__file__).resolve().parent;NATIVE=HERE if(HERE/'breadcrumbs-render.rs').is_file()else HERE.parents[1]
def apply(edit,once):
 renderer=runpy.run_path(str(HERE/'potion-fx-render.py'))
 for file in ['potion-fx-game.rs','potion-fx-ground.rs']:
  edit('crates/horizon-client/src/game/'+file.replace('-game','').replace('-ground','_ground').replace('-','_'),lambda _,file=file:(HERE/file).read_text(encoding='utf8'))
 edit('crates/horizon-client-render/src/gfx/potion_fx.rs',lambda _:renderer['source']())
 edit('crates/horizon-client-render/src/gfx/potion-fx.wgsl',lambda _:(HERE/'potion-fx.wgsl').read_text(encoding='utf8'))
 edit('crates/horizon-client-render/src/gfx/potion-fx-textures.rs',lambda _:renderer['textures']())
 def game(t):
  t=once(t,'mod target_renderer;','mod target_renderer;\nmod potion_fx;\nmod potion_fx_ground;')
  t=once(t,'    potions: potions::PotionState,','    potions: potions::PotionState,\n    liquidbounce_potion_fx:potion_fx::NativeState,')
  t=once(t,'            potions: Default::default(),','            potions: Default::default(),\n            liquidbounce_potion_fx:Default::default(),')
  return once(t,'        horizon_client::gfx::breadcrumbs::begin_frame("null");','        horizon_client::gfx::breadcrumbs::begin_frame("null");\n        self.client_potion_fx_reset();\n        horizon_client::gfx::potion_fx::begin_frame("null");')
 edit('crates/horizon-client/src/game.rs',game)
 def hooks(t):
  t=once(t,'    pub(super) fn native_client_tick_pre(&mut self) {','    pub(super) fn native_client_tick_pre(&mut self) {\n        self.client_potion_fx_errors();')
  anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
  return once(t,anchor,'    pub(super) fn native_client_potion_fx(&self,input:&str,geometry:&mut String){if self.native_window(){mod_api::dispatch_client_potion_fx(self,mod_api::mods(),input,geometry);}}\n\n'+anchor)
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 def remote(t):
  t=once(t,'        self.native_client_packet(&packet);','        self.native_client_packet(&packet);\n        self.client_potion_fx_packet(&packet);')
  return once(t,'    fn tick_remote_entities(&mut self, running: bool) {','    fn tick_remote_entities(&mut self, running: bool) {\n        if running{self.client_potion_fx_tick();}')
 edit('crates/horizon-client/src/game/remote.rs',remote)
 def clouds(t):
  t=once(t,'    particle: ParticleOptions,','    particle: ParticleOptions,\n    liquidbounce_potion_fx_age:i32,')
  t=once(t,'                particle: default_cloud_particle(),','                particle: default_cloud_particle(),\n                liquidbounce_potion_fx_age:0,')
  t=once(t,'        for c in &self.potions.clouds {\n            cloud_client_tick','        for c in &mut self.potions.clouds {\n            c.liquidbounce_potion_fx_age=c.liquidbounce_potion_fx_age.wrapping_add(1);\n            cloud_client_tick')
  return t+'''
impl Game<'_>{
    pub(super)fn client_potion_fx_clouds(&self)->Vec<serde_json::Value>{
        let pack=|p:&ParticleOptions|->Option<i32>{
            // ColorParticleOption used by actual potion clouds. Other option types
            // cannot supply the source accessor's integer color.
            if !matches!(p.ty,ParticleType::EntityEffect|ParticleType::Flash|ParticleType::TintedLeaves){return None;}
            let byte=|i:usize|((p.color[i]*255.).floor()as u32)&255;
            Some(((byte(3)<<24)|(byte(0)<<16)|(byte(1)<<8)|byte(2))as i32)
        };
        let mut out=vec![];
        for c in &self.potions.clouds{if let Some(color)=pack(&c.particle){out.push(serde_json::json!({"position":c.at,"radius":c.radius,"age":c.liquidbounce_potion_fx_age,"color":color}));}}
        for c in &self.breath_clouds{if let Some(potion)=&c.potion&&let Some(color)=pack(&potion.particle()){out.push(serde_json::json!({"position":c.at,"radius":c.radius,"age":c.age,"color":color}));}}
        out
    }
}
'''
 edit('crates/horizon-client/src/game/remote_projectiles.rs',clouds)
 edit('crates/horizon-client/src/game/world_primitives.rs',lambda t:once(t,'        self.client_target_renderer_frame(alpha);','        self.client_target_renderer_frame(alpha);\n        self.client_potion_fx_frame(alpha);'))
 def gfx(t):
  t=once(t,'pub mod target_renderer;','pub mod target_renderer;\npub mod potion_fx;')
  t=once(t,'    target_renderer:target_renderer::TargetRenderer,','    target_renderer:target_renderer::TargetRenderer,\n    potion_fx:potion_fx::PotionFxRenderer,')
  t=once(t,'        let target_renderer=target_renderer::TargetRenderer::new(&device,frame_format,linear_light);','        let target_renderer=target_renderer::TargetRenderer::new(&device,frame_format,linear_light);\n        let potion_fx=potion_fx::PotionFxRenderer::new(&device,frame_format,linear_light);')
  t=once(t,'            target_renderer,','            target_renderer,\n            potion_fx,')
  anchor='        self.target_renderer.prepare(&self.device,&self.queue,jittered,frame.camera,frame.title);'
  t=once(t,anchor,anchor+'\n        self.potion_fx.prepare(&self.device,&self.queue,jittered,frame.camera,frame.title);')
  t=once(t,'        let hand_in_world = ','        let hand_in_world = !self.potion_fx.pending() && ')
  anchor='        self.target_renderer.draw(&self.device,&mut encoder,world_view,world_depth);'
  return once(t,anchor,anchor+'\n        self.potion_fx.draw(&mut encoder,world_view,world_depth);')
 edit('crates/horizon-client-render/src/gfx.rs',gfx)
 return {'hook_names':['client_potion_fx'],'helper_names':[]}
