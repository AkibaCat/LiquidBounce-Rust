"""Actual HitFX packet/particle/self-hurt/audio integration; embedded original Ogg assets."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/hit_fx.rs',lambda _:(HERE/'hit-fx-game.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game.rs',lambda t:once(t,'mod particles;','mod particles;\nmod hit_fx;'))
 anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda t:once(t,anchor,(HERE/'hit-fx-host.rs').read_text(encoding='utf8')+'\n'+anchor))
 def remote(t):
  a='''            C::Sound(p) => {
                if let Some(name) = self.remote_sound_name(&p.sound)
                    && !self.pvp_mutes_sound(&name)'''
  b='''            C::Sound(p) => {
                if let Some(name) = self.remote_sound_name(&p.sound)
                    && !self.client_hit_fx_packet(&name,p.source,[f64::from(p.x)/8.,f64::from(p.y)/8.,f64::from(p.z)/8.],p.volume,p.pitch,p.seed)
                    && !self.pvp_mutes_sound(&name)'''
  t=once(t,a,b)
  a='''                        let sound = self.remote_hurt_sound(p.source_type);
                        let pitch = player_voice_pitch();
                        self.audio.play_on(
                            horizon_client::settings::sound_source(sound),
                            sound,'''
  b='''                        let source = horizon_client::settings::sound_source(self.remote_hurt_sound(p.source_type));
                        let sound = self.client_hit_fx_self_sound(self.remote_hurt_sound(p.source_type));
                        let pitch = player_voice_pitch();
                        self.audio.play_on(
                            source,
                            &sound,'''
  return once(t,a,b)
 edit('crates/horizon-client/src/game/remote.rs',remote)
 edit('crates/horizon-client-render/src/hit-fx-audio-data.rs',lambda _:(HERE/'hit-fx-audio-data.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client-render/src/hit-fx-sounds.json',lambda _:(HERE/'hit-fx-sounds.json').read_text(encoding='utf8'))
 def audio(t):
  t=once(t,'    let file = fs::File::open(path).ok()?;\n    let stream = MediaSourceStream::new(Box::new(file), Default::default());','''    let file:Box<dyn symphonia::core::io::MediaSource>=if let Some(data)=embedded_hit_fx(path){Box::new(std::io::Cursor::new(data))}else{Box::new(fs::File::open(path).ok()?)};
    let stream = MediaSourceStream::new(file, Default::default());''')
  a='    /// Play a sound on a given `SoundSource` index (`/playsound`).\n'
  t=once(t,a,'''    /// Seeded client-only sound with the packet's original source, volume and pitch.
    pub fn play_on_seeded(&mut self,source:usize,event:&str,at:Option<[f64;3]>,volume:f32,pitch:f32,seed:i64){self.source_override=Some(source);self.play_seeded(event,at,volume,pitch,seed);self.source_override=None;}
'''+a)
  return t+'\ninclude!("hit-fx-audio-data.rs");\n'
 edit('crates/horizon-client-render/src/audio.rs',audio)
 edit('crates/horizon-client-render/src/sound_events.rs',lambda t:once(t,'        registry\n    }\n    /// No sound event at all:', '        registry.add_file("liquidbounce",include_bytes!("hit-fx-sounds.json"),&|_,_|true);\n        registry\n    }\n    /// No sound event at all:'))
 return {'hook_names':['client_hit_fx'],'helper_names':['client_hit_fx_context']}
