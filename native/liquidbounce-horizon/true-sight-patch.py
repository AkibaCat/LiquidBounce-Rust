from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/true_sight.rs',lambda _:(HERE/'true-sight-render.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/client_lib.rs',lambda t:once(t,'pub mod block_contact;\n','pub mod block_contact;\npub mod true_sight;\n'))
 anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda t:once(t,anchor,(HERE/'true-sight-host.rs').read_text(encoding='utf8')+'\n'+anchor))
 def mobs(t):
  t=once(t,'            let outline = self.mob_outline(mob);\n','            let true_sight=self.client_true_sight_colors(mob.id as i32);\n            let _true_sight_scope=horizon_client::true_sight::scope(true_sight);\n            let outline = self.mob_outline(mob);\n')
  t=once(t,'            let layers_only = mob.invisible && outline.is_none() && is_player;','            let layers_only = mob.invisible && true_sight.is_none() && outline.is_none() && is_player;')
  t=once(t,'            if mob.invisible && outline.is_none() && !layers_only {','            if mob.invisible && true_sight.is_none() && outline.is_none() && !layers_only {')
  return once(t,'            let posable = skinning\n','            let posable = skinning && true_sight.is_none()\n')
 edit('crates/horizon-client/src/game/creatures/mob_geometry.rs',mobs)
 def render(t):
  t=once(t,'    if super::trader::is_invisible(m) {\n        if m.species() == Species::Illusioner {','    if super::trader::is_invisible(m) && !crate::true_sight::active() {\n        if m.species() == Species::Illusioner {')
  t=once(t,'        Some(skin) => {\n            custom_body.is_none()','        Some(skin) => {\n            !crate::true_sight::active() && custom_body.is_none()')
  assert t.count('                body_color(m, alpha),')==2
  t=t.replace('                body_color(m, alpha),','                crate::true_sight::body(body_color(m, alpha)),')
  t=once(t,'            if translucent_body(m) {\n                mark_oit(&mut out[start..]);','            if translucent_body(m) || crate::true_sight::active() {\n                mark_oit(&mut out[start..]);')
  # This is RenderLayer.renderColoredCutoutModel, excluding equipment, eyes,
  # energy swirl and the breeze wind's custom render types.
  t=once(t,'        let first_job = skin.as_deref().map(|skin| skin.frame.jobs.len());\n','        let true_sight_feature=crate::true_sight::active() && armor_slot(&layer.model).is_none() && !layer.full_bright && layer.model!="breeze#wind";\n        let layer_color=if true_sight_feature {crate::true_sight::feature(layer.color)}else{layer.color};\n        let first_job = skin.as_deref().map(|skin| skin.frame.jobs.len());\n')
  assert t.count('            layer.color,')==2;t=t.replace('            layer.color,','            layer_color,')
  return once(t,'        if translucent_layer(m, layer) {\n            mark_oit(&mut out[start..]);','        if translucent_layer(m, layer) || true_sight_feature {\n            mark_oit(&mut out[start..]);')
 edit('crates/horizon-client/src/mobs/render.rs',render)
 def player(t):
  t=once(t,'        let body = self.own_body_visible();','        let true_sight=self.client_true_sight_colors(self.own_entity_id());\n        let _true_sight_scope=horizon_client::true_sight::scope(true_sight);\n        let body = self.own_body_visible() || true_sight.is_some();')
  return t
 edit('crates/horizon-client/src/game/perspective.rs',player)
 def stands(t):
  anchor='            let culling = self.entity_cull_decision(a, alpha);\n'
  t=once(t,anchor,'            let true_sight=self.client_true_sight_colors(a.id as i32);\n            let _true_sight_scope=horizon_client::true_sight::scope(true_sight);\n'+anchor)
  start=t.index('    pub(super) fn armor_stand_geometry(');end=t.index('    /// `ArmorStandRenderer.submit`',start)
  body=t[start:end];body=once(body,'            let skinned = if skinning {','            let skinned = if skinning && true_sight.is_none() {');t=t[:start]+body+t[end:]
  start=t.index('    fn armor_stand_vertices(');end=t.index('        self.armor_stand_items(',start);body=t[start:end]
  body=once(body,'        if !a.invisible {','        let true_sight_body_start=out.len();\n        if !a.invisible || horizon_client::true_sight::active() {')
  assert body.count('                0xFFFF_FFFF,')==2;body=body.replace('                0xFFFF_FFFF,','                horizon_client::true_sight::body(0xFFFF_FFFF),')
  return t[:start]+body+'        horizon_client::true_sight::mark(&mut out[true_sight_body_start..]);\n'+t[end:]
 edit('crates/horizon-client/src/game/armor_stands.rs',stands)
 def tick(t):
  slot='self.client_hotbar_slot()' if 'let marker = marker_particle_target(self.mode, self.stack_in(self.client_hotbar_slot()));' in t else 'self.selected'
  anchor=f'        let marker = marker_particle_target(self.mode, self.stack_in({slot}));\n'
  return once(t,anchor,'        let mut true_sight_markers=0;self.native_client_true_sight_markers(&mut true_sight_markers);\n        let _true_sight_markers_scope=horizon_client::true_sight::marker_scope(true_sight_markers);\n        let marker = if true_sight_markers!=0 {Some("minecraft:barrier")}else{marker_particle_target(self.mode,self.stack_in('+slot+'))};\n')
 edit('crates/horizon-client/src/game/render_tick.rs',tick)
 edit('crates/horizon-client/src/animate_tick.rs',lambda t:once(t,'ctx.registry.state(state).is_some_and(|s| s.name == block)','ctx.registry.state(state).is_some_and(|s| crate::true_sight::marker_matches(s.name == block, &s.name))'))
 return {'hook_names':['client_true_sight_entity','client_true_sight_markers'],'helper_names':[]}
