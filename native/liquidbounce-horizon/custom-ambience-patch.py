"""All CustomAmbience mixin phases and actual original builtin particle quads."""
from pathlib import Path
import base64
HERE=Path(__file__).resolve().parent
NATIVE=HERE if(HERE/'particles-patch.py').is_file()else HERE.parents[1]
ROOT=NATIVE.parents[1]
ASSETS=list(zip(['Orbiz','Star','Dollar','Crown','Heart','Lightning','Line','Point','Rhombus','Snowflake','Spark'],['glow','star','dollar','crown','heart','lightning','line','point','rhombus','snowflake','spark']))
def textures():
 root=ROOT/'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/particles'
 return 'const PNGS:[(&str,&str);11]=[\n'+''.join('("'+name+'","'+base64.b64encode((root/(file+'.png')).read_bytes()).decode()+'"),\n'for name,file in ASSETS)+'];\n'
def apply(edit,once):
 for dst,src in [('crates/horizon-client/src/game/custom_ambience.rs','custom-ambience-game.rs'),('crates/horizon-client-render/src/gfx/custom_ambience.rs','custom-ambience-render.rs'),('crates/horizon-client-render/src/gfx/custom-ambience.wgsl','custom-ambience.wgsl')]:edit(dst,lambda _,src=src:(HERE/src).read_text(encoding='utf8'))
 edit('crates/horizon-client-render/src/gfx/custom-ambience-textures.rs',lambda _:textures())
 def game(t):
  t=once(t,'mod target_renderer;','mod target_renderer;\nmod custom_ambience;')
  t=once(t,'    fn render_frame(&mut self) -> Result<bool> {','    fn render_frame(&mut self) -> Result<bool> {\n        self.client_custom_ambience_refresh();')
  t=once(t,'        pr.lap("pr_environment");','        let ambience_fog_sky=sky;\n        self.client_custom_ambience_sky(&mut sky);\n        pr.lap("pr_environment");')
  t=once(t,'fog: Some(self.frame_fog(camera, &sky, biome, water_fog)),','fog: Some(self.frame_fog(camera, &ambience_fog_sky, biome, water_fog)),')
  a='        horizon_client::gfx::breadcrumbs::begin_frame("null");'
  return once(t,a,a+'\n        horizon_client::gfx::custom_ambience::begin_frame("null");\n        if !title{self.client_custom_ambience_frame(alpha);}')
 edit('crates/horizon-client/src/game.rs',game)
 def hooks(t):
  a='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
  t=once(t,a,'    pub(super)fn native_client_custom_ambience(&self,input:&str,output:&mut String){if self.native_window(){mod_api::dispatch_client_custom_ambience(self,mod_api::mods(),input,output);}}\n\n'+a)
  for anchor in ['    pub(super) fn native_client_tick_pre(&mut self) {','    pub(super) fn native_frame(&mut self, elapsed: Duration) {']:
   t=once(t,anchor,anchor+'\n        self.client_custom_ambience_refresh();')
  a='''    pub(super) fn native_day_time(&self) -> i64 {
        self.world.day_time
    }'''
  t=once(t,a,a.replace('self.world.day_time','self.client_custom_ambience_clock(self.world.day_time,"minecraft:overworld")'))
  a='''        self.player.is_some().then_some(NativeClock {
            game_time: self.world.game_time,
            day_time: self.world.day_time,
        })'''
  return once(t,a,a.replace('day_time: self.world.day_time','day_time: self.client_custom_ambience_time(self.world.day_time)'))
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 def weather(t):
  a='''        if self.can_have_weather() {
            self.world.weather.levels(partial)
        } else {
            (0.0, 0.0)
        }'''
  t=once(t,a,'        self.client_custom_ambience_weather('+a.strip()+')')
  # EnvironmentAttributeSystem samples a ClockInstance; this is client sky brightness.
  return t.replace('self.world.day_time','self.client_custom_ambience_time(self.world.day_time)')
 edit('crates/horizon-client/src/game/weather.rs',weather)
 def pvp(t):
  a='        horizon_client::pvp::view_day_time(self.world.day_time, fixed.filter(|_| self.own_view()))'
  t=once(t,a,'        self.client_custom_ambience_time('+a.strip()+')')
  return once(t,'Some(w) if self.own_view() && self.can_have_weather() => w.levels(),','Some(w) if self.own_view() && self.can_have_weather() => self.client_custom_ambience_weather(w.levels()),')
 edit('crates/horizon-client/src/game/extras/pvp.rs',pvp)
 def item_time(t):
  return t.replace('self.world.day_time','self.client_custom_ambience_time(self.world.day_time)')
 edit('crates/horizon-client/src/game/item_draw.rs',item_time)
 def level_time(t):
  a='''    pub(super) fn native_level_day_time(&self, level: LevelRef) -> Option<i64> {
        self.native_level_world(level)?;
        Some(self.world.day_time)
    }'''
  return once(t,a,a.replace('Some(self.world.day_time)','Some(self.client_custom_ambience_clock(self.world.day_time,"minecraft:overworld"))'))
 edit('crates/horizon-client/src/game/native_level_refs.rs',level_time)
 # Read-only visual consumers of the client world's default ClockInstance.
 for path in ['creatures/mob_geometry.rs','debug_lines.rs']:
  edit('crates/horizon-client/src/game/'+path,item_time)
 def random_textures(t):
  t=item_time(t)
  t=once(t,'        let weather = &self.world.weather;','        let (rain,thunder)=self.weather_levels(1.0);')
  t=once(t,'weather: if weather.thundering {','weather: if thunder>0.9 {')
  return once(t,'} else if weather.raining {','} else if rain>0.2 {')
 edit('crates/horizon-client/src/game/random_textures.rs',random_textures)
 def shader_inputs(t):
  a='''            day_time: if dimension.is_some_and(|d| d.default_clock.is_none()) {
                0
            } else {
                self.world.day_time
            },'''
  t=once(t,a,a.replace('self.world.day_time','self.client_custom_ambience_time(self.world.day_time)'))
  t=once(t,'            rain: lerp(self.world.weather.rain_level),','            rain: self.weather_levels(partial).0,')
  return once(t,'            thunder: lerp(self.world.weather.thunder_level),','            thunder: self.weather_levels(partial).1,')
 edit('crates/horizon-client/src/game/shader_packs.rs',shader_inputs)
 def precipitation(t):
  a='        let (intensity, _) = self.view_weather(alpha as f32);'
  t=once(t,a,a+'\n        let (intensity,force_snow)=self.client_custom_ambience_precipitation(intensity);')
  return once(t,'                let Some(snow) = self.precipitation(x, cy, z) else {','                let Some(snow) = (if force_snow{Some(true)}else{self.precipitation(x, cy, z)}) else {')
 edit('crates/horizon-client/src/game/frame_geometry.rs',precipitation)
 def fog(t):
  # Client particle/sound environment attributes read the same default clock.
  a='(12600..23401).contains(&self.world.day_time.rem_euclid(24000));'
  assert t.count(a)==2
  t=t.replace(a,'(12600..23401).contains(&self.client_custom_ambience_time(self.world.day_time).rem_euclid(24000));')
  t=once(t,'let thundering = self.can_have_weather() && world.weather.levels(1.0).1 > 0.9;','let thundering = self.can_have_weather() && self.weather_levels(1.0).1 > 0.9;')
  a='''            fog = self.native_fog_now(self.clock.fraction() as f32, at, fog_type, fog);
        }
        fog'''
  return once(t,a,a[:-3]+'self.client_custom_ambience_fog(fog)')
 edit('crates/horizon-client/src/game/render_tick.rs',fog)
 def gfx(t):
  t=once(t,'pub mod target_renderer;','pub mod target_renderer;\npub mod custom_ambience;')
  t=once(t,'    target_renderer:target_renderer::TargetRenderer,','    target_renderer:target_renderer::TargetRenderer,\n    custom_ambience:custom_ambience::CustomAmbienceRenderer,')
  a='        let target_renderer=target_renderer::TargetRenderer::new(&device,frame_format,linear_light);'
  t=once(t,a,a+'\n        let custom_ambience=custom_ambience::CustomAmbienceRenderer::new(&device,frame_format,linear_light);')
  t=once(t,'            target_renderer,','            target_renderer,\n            custom_ambience,')
  a='        self.target_renderer.prepare(&self.device,&self.queue,jittered,frame.camera,frame.title);'
  t=once(t,a,a+'\n        self.custom_ambience.prepare(&self.device,&self.queue,jittered,frame.camera,frame.title);')
  t=once(t,'        let hand_in_world = ','        let hand_in_world = !self.custom_ambience.pending() && ')
  a='        self.target_renderer.draw(&self.device,&mut encoder,world_view,world_depth);'
  t=once(t,a,a+'\n        self.custom_ambience.draw(&mut encoder,world_view,world_depth);')
  t=once(t,'        let clear = linear_blend::stored_clear(clear, self.output.frame.linear_light);','        let clear=custom_ambience::clear_color(clear);\n        let clear = linear_blend::stored_clear(clear, self.output.frame.linear_light);')
  a='        // lightmap.fsh: per (block, sky) level.'
  t=once(t,a,'''        let mut sky=*sky;
        let rgb=custom_ambience::lightmap_color;
        if let Some(value)=rgb("block"){sky.block_light_tint=value;}
        if let Some(value)=rgb("sky"){sky.sky_light_color=value;}
        if let Some(value)=rgb("ambient"){sky.ambient_light_color=value;}
        let night_color=Vec3::from_array(rgb("night").unwrap_or([0.6;3]));
'''+a)
  t=once(t,'Vec3::from(sky.ambient_light_color).max(Vec3::splat(0.6 * night_vision));','Vec3::from(sky.ambient_light_color).max(night_color * night_vision);')
  a='''        if self.fullbright {
            pixels.fill(255);
        }'''
  return once(t,a,a+'\n        custom_ambience::lightmap_fill(&mut pixels);')
 edit('crates/horizon-client-render/src/gfx.rs',gfx)
 return {'hook_names':['client_custom_ambience'],'helper_names':[]}
