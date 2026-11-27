"""Complete TNTTimer and reusable original indicator layout/SDF native consumer."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 for dst,src in [('crates/horizon-client/src/game/indicator.rs','indicator-game.rs'),('crates/horizon-client/src/game/indicator_math.rs','indicator.rs'),('crates/horizon-client/src/game/tnt_timer.rs','tnt-timer-game.rs'),('crates/horizon-client-render/src/gfx/indicator.rs','indicator-render.rs'),('crates/horizon-client-render/src/gfx/indicator.wgsl','indicator.wgsl')]:
  source=HERE/src
  if not source.is_file()and src=='indicator.rs':source=HERE.parent/'liquidbounce-behaviors/src/indicator.rs'
  edit(dst,lambda _,source=source:source.read_text(encoding='utf8'))
 def game(t):
  t=once(t,'mod target_renderer;','mod target_renderer;\nmod indicator;\nmod tnt_timer;')
  t=once(t,'            self.gui_layer("minecraft:hotbar", &mut batch, |g, batch| {','            self.client_tnt_timer_submit(&mut batch);\n            self.client_indicator_overlay(&mut batch);\n            self.gui_layer("minecraft:hotbar", &mut batch, |g, batch| {')
  return once(t,'        let mut hud = horizon_client::spans::time("hud", || self.hud(size));','        indicator::clear();\n        let mut hud = horizon_client::spans::time("hud", || self.hud(size));')
 edit('crates/horizon-client/src/game.rs',game)
 def hooks(t):
  a='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
  return once(t,a,'    pub(super)fn native_client_tnt_timer(&self,input:&str,output:&mut String){if self.native_window(){mod_api::dispatch_client_tnt_timer(self,mod_api::mods(),input,output);}}\n\n'+a)
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 def tnt(t):
  start=t.index('    pub(super) fn tnt_geometry(');end=t.index('\n}\n\nimpl Game',start);body=t[start:end]
  body=once(body,'            self.mark_motion(out.len(), Self::MOTION_OBJECT | u64::from(t.id));','            let first=out.len();\n            self.mark_motion(out.len(), Self::MOTION_OBJECT | u64::from(t.id));')
  a='            }\n        }\n    }';assert body.endswith(a)
  body=body[:-len(a)]+'''            }
            if let Some(color)=self.client_tnt_timer_glow(t.fuse).or_else(||self.thing_outline(t.id,"minecraft:tnt")){self.mark_outline(&out[first..],color);}
        }
    }'''
  return t[:start]+body+t[end:]
 edit('crates/horizon-client/src/game/tnt.rs',tnt)
 def gfx(t):
  t=once(t,'pub mod target_renderer;','pub mod target_renderer;\npub mod indicator;')
  t=once(t,'    target_renderer:target_renderer::TargetRenderer,','    target_renderer:target_renderer::TargetRenderer,\n    indicator:indicator::IndicatorRenderer,')
  t=once(t,'        let mut hud = HudRenderer::new(','        let indicator=indicator::IndicatorRenderer::new(&device,overlay_format,hud_coverage);\n        let mut hud = HudRenderer::new(')
  t=once(t,'            target_renderer,','            target_renderer,\n            indicator,')
  t=once(t,'        self.hud.prepare(','        self.indicator.prepare(&self.device,frame.title);\n        self.hud.prepare(')
  for first,last,part in [('0','blur_at','before'),('blur_at','frame.hud.vertices.len() as u32','after')]:
   call=f'''        self.hud.draw_range(
            encoder,
            hud_target,
            coverage.as_ref(),
            &self.depth,
            frame.hud,
            [self.config.width, self.config.height],
            {first}..{last},
            timing.as_mut(),
        );'''
   limit=f'let indicator_at=self.indicator.layer_end().unwrap_or(u32::MAX);let indicator_cut=indicator_at.clamp({first},{last});'
   condition='indicator_at<=blur_at'if part=='before'else'indicator_at>blur_at&&indicator_at<=frame.hud.vertices.len()as u32'
   if part=='after':
    # Keep the original final draw call intact: the independent GPU UI patch
    # appends its extension immediately after it. Insert our split before it.
    prefix=call.replace(f'{first}..{last}',f'{first}..indicator_cut')
    prefix='\n'.join('    '+line for line in prefix.splitlines())
    replacement='        '+limit+'\n        if indicator_cut>blur_at {\n'+prefix+'\n        }\n'+f'        if {condition}{{self.indicator.draw(encoder,hud_target,coverage.as_ref());}}\n'+'        let blur_at=indicator_cut;\n'+call
   else:
    replacement='        '+limit+'\n'+call.replace(f'{first}..{last}',f'{first}..indicator_cut')+f'\n        if {condition}{{self.indicator.draw(encoder,hud_target,coverage.as_ref());}}\n'+call.replace(f'{first}..{last}',f'indicator_cut..{last}')
   t=once(t,call,replacement)
  return t
 edit('crates/horizon-client-render/src/gfx.rs',gfx)
 return {'hook_names':['client_tnt_timer'],'helper_names':[]}
