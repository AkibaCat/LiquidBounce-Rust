"""DamageParticles: true health-setter capture, all-living tick snapshots and native HUD text."""
from pathlib import Path
import re
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/damage_particles_capture.rs',lambda _:(HERE/'damage-particles-capture.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/client_lib.rs',lambda t:once(t,'pub mod health;','pub mod health;\npub mod damage_particles_capture;'))
 edit('crates/horizon-client/src/game/damage_particles.rs',lambda _:(HERE/'damage-particles-game.rs').read_text(encoding='utf8'))
 def world(t):
  t=once(t,'pub fn set_health<W: HealthWorld>', '/// Before a real LivingEntity.setHealth storage operation; entity context belongs to its caller.\npub static HEALTH_SET_OBSERVER:std::sync::OnceLock<fn(f32,f32,f32)>=std::sync::OnceLock::new();\npub fn set_health<W: HealthWorld>')
  return once(t,'    world.store_health(data, clamp(value, maximum))','    if let Some(observer)=HEALTH_SET_OBSERVER.get(){let old=world.health()?;let new=clamp(value,maximum);if old!=new{observer(old,new,maximum);}}\n    world.store_health(data, clamp(value, maximum))')
 edit('crates/horizon-world/src/living_health.rs',world)
 # These six actual player health entry boundaries cover ordinary, equipment, food and regeneration setters.
 for file,count in [('health.rs',2),('hunger.rs',2),('health/equipment.rs',2)]:
  def scopes(t,count=count):
   old='let h = &mut self.health;'
   assert t.count(old)==count,(file,t.count(old))
   return t.replace(old,'let _damage_particles_scope=crate::damage_particles_capture::player(self);\n        '+old)
  edit('crates/horizon-client/src/'+file,scopes)
 def setters(t):
  # Some files have cfg(test) imports before production. Apply the reviewed living-type
  # receivers throughout each selected file; incoming entity metadata files are excluded.
  pattern=r'(?m)^(\s*)(self|m|child|alive|attacker|wearer|d|self\.armor_stands\[i\]|self\.mobs\[v\])\.health\s*(=|-=)\s*([^;]+);'
  def replacement(m):
   indent,obj,op,value=m.groups()
   if '\n' in value and not value.lstrip().startswith('if '):return m.group(0)
   if op=='-=':value=obj+'.health - ('+value+')'
   return f'{indent}{obj}.client_lb_set_health({value});'
  return re.sub(pattern,replacement,t)
 for file in ['mobs.rs','mobs/finalize.rs','mobs/creaking.rs','mobs/creaking_heart.rs','mobs/cube.rs','mobs/sulfur_cube.rs','mobs/equine.rs','mobs/llama.rs','mobs/panda.rs','mobs/tame.rs','mobs/wither.rs','game/creatures.rs','game/mounts.rs','game/native_mobs.rs','game/commands.rs','game/mob_blocks.rs','game/zombie_villagers.rs','game/armor_stands.rs','game/remote.rs']:
  edit('crates/horizon-client/src/'+file,setters)
 def stands(t):
  for value in ['4.0','0.15']:t=once(t,f'self.health -= {value};',f'self.client_lb_set_health(self.health - {value});')
  return t
 edit('crates/horizon-client/src/entities.rs',stands)
 edit('crates/horizon-client/src/game/dragon.rs',lambda t:setters(t)+'\n'+(HERE/'damage-particles-dragon.rs').read_text(encoding='utf8'))
 def game(t):
  t=once(t,'mod breadcrumbs;','mod breadcrumbs;\nmod damage_particles;')
  # HEAD of extractHotbarAndDecorations: the original event is after health and before contextual bars.
  return once(t,'            // After the health rows (Hud.extractHotbarAndDecorations): jump, locator or XP bar.','            self.client_damage_particles_overlay(&mut batch);\n            // After the health rows (Hud.extractHotbarAndDecorations): jump, locator or XP bar.')
 edit('crates/horizon-client/src/game.rs',game)
 def hooks(t):
  t=once(t,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',(HERE/'damage-particles-host.rs').read_text(encoding='utf8')+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n')
  t=once(t,'        mod_api::dispatch_client_tick_pre(self, mod_api::mods());','        self.client_damage_particles_tick();\n        mod_api::dispatch_client_tick_pre(self, mod_api::mods());')
  return once(t,'        self.client_breadcrumbs_reset();','        self.client_damage_particles_reset();\n        self.client_breadcrumbs_reset();')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 def lan(t):
  t=once(t,'        let known_movement = std::mem::replace(&mut self.known_movement, Vec3::new(x, y, z));','        let known_movement = std::mem::replace(&mut self.known_movement, Vec3::new(x, y, z));\n        horizon_client::damage_particles_capture::actor_movement(Some(self.known_movement));')
  return once(t,'        self.known_movement = known_movement;','        self.known_movement = known_movement;\n        horizon_client::damage_particles_capture::actor_movement(None);')
 edit('crates/horizon-client/src/game/lan_actions.rs',lan)
 return {'hook_names':['client_damage_particles'],'helper_names':[]}
