#!/usr/bin/env python3
"""Complete Chams capture/composite. Apply after Hitbox and camera-control."""
from pathlib import Path

HERE=Path(__file__).resolve().parent
HOOK_NAMES=['client_chams_settings','client_chams_entity']
HELPER_NAMES=[]
HOOKS='''    /// ModuleChams.beginFrame: a live render configuration, or null while disabled.
    pub(super) fn native_client_chams_settings(&self, settings:&mut String) {
        if self.native_window() {
            mod_api::dispatch_client_chams_settings(self,mod_api::mods(),settings);
        }
    }
    /// Full Visual Targets selection; raw F5 perspective is distinct from AutoF5.
    pub(super) fn native_client_chams_entity(&self,entity:i32,raw_perspective:u8,target:&mut bool) {
        if self.native_window() {
            mod_api::dispatch_client_chams_entity(self,mod_api::mods(),entity,raw_perspective,target);
        }
    }
    pub(super) fn client_chams_entity(&self,entity:i32)->bool {
        if !horizon_client::gfx::chams::active() {return false;}
        let mut target=false;self.native_client_chams_entity(entity,self.native_raw_camera_perspective(),&mut target);target
    }

'''

def apply(edit,once):
    edit('crates/horizon-client-render/src/gfx/chams.rs',lambda _:(HERE/'chams-render.rs').read_text(encoding='utf-8'))
    edit('crates/horizon-client-render/src/gfx/chams.wgsl',lambda _:(HERE/'chams.wgsl').read_text(encoding='utf-8'))
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda t:once(t,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',HOOKS+'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
    def game(t):
        t=once(t,'        let mut entities = std::mem::take(&mut self.entity_vertices);\n',
            '''        let mut chams_settings="null".to_owned();
        if !title {self.native_client_chams_settings(&mut chams_settings);}
        horizon_client::gfx::chams::begin_frame(&chams_settings);
        let mut entities = std::mem::take(&mut self.entity_vertices);
''')
        return once(t,'        let entity_spans = std::mem::take(self.entity_spans.get_mut());\n',
            '''        // The original Chams storage removes captures from the ordinary pass.
        // Rebase the identities that were recorded before this final separation.
        let chams_offsets=horizon_client::gfx::chams::finish_entities(&mut entities);
        if !chams_offsets.is_empty() {
            for span in self.entity_spans.get_mut().iter_mut() {
                span.start=chams_offsets[span.start as usize];
            }
            for (start,_) in self.entity_motion.get_mut().iter_mut() {
                *start=chams_offsets[*start as usize];
            }
        }
        let entity_spans = std::mem::take(self.entity_spans.get_mut());
''')
    edit('crates/horizon-client/src/game.rs',game)
    def skin(t):
        t=once(t,'        let renderer = self.renderer.as_ref().filter(|r| r.skins_entities());\n',
            '        let renderer = self.renderer.as_ref().filter(|r| r.skins_entities() && !horizon_client::gfx::chams::active());\n')
        return once(t,'''    pub(super) fn skinning(&self) -> bool {
        self.renderer''','''    pub(super) fn skinning(&self) -> bool {
        !horizon_client::gfx::chams::active() && self.renderer''')
    edit('crates/horizon-client/src/game/creatures.rs',skin)
    def mobs(t):
        t=once(t,'            let outline = self.mob_outline(mob);\n','            let chams=self.client_chams_entity(mob.id as i32);\n            let outline = self.mob_outline(mob);\n')
        t=once(t,'            if culling.hides_entity() {\n','            if !chams && culling.hides_entity() {\n')
        t=once(t,'                if outline.is_none() {\n                    self.cull_entity_vertices(out, entity_start);\n',
            '                if chams {horizon_client::gfx::chams::mark_entity(&mut out[entity_start..]);}\n                if !chams && outline.is_none() {\n                    self.cull_entity_vertices(out, entity_start);\n')
        t=once(t,'            if outline.is_none() && culling.checks_vertices() && beam.is_none() {\n',
            '            if chams {horizon_client::gfx::chams::mark_entity(&mut out[entity_start..]);}\n            if !chams && outline.is_none() && culling.checks_vertices() && beam.is_none() {\n')
        return t
    edit('crates/horizon-client/src/game/creatures/mob_geometry.rs',mobs)
    def stands(t):
        anchor='            let culling = self.entity_cull_decision(a, alpha);\n'
        t=once(t,anchor,'            let chams=self.client_chams_entity(a.id as i32);\n'+anchor)
        # Restrict to ArmorStandRenderer, keeping unrelated object culling intact.
        start=t.index('    pub(super) fn armor_stand_geometry(')
        end=t.index('    /// `ArmorStandRenderer.submit`',start)
        body=t[start:end]
        body=once(body,'            if culling.hides_entity() {\n','            if !chams && culling.hides_entity() {\n')
        body=once(body,'            if !self.entity_glowing(a.id) {\n',
            '            if chams {horizon_client::gfx::chams::mark_entity(&mut out[entity_start..]);}\n            if !chams && !self.entity_glowing(a.id) {\n')
        return t[:start]+body+t[end:]
    edit('crates/horizon-client/src/game/armor_stands.rs',stands)
    def player(t):
        start=t.index('    pub(super) fn player_model_geometry(')
        end=t.index('    /// `extractEntityInInventoryFollowsMouse`',start)
        body=t[start:end]
        body=once(body,'        let start = out.len();\n','        let start = out.len();\n        let chams=self.client_chams_entity(self.own_entity_id());\n')
        body=once(body,'        if body && self.mod_player_geometry(m, who, alpha as f32, camera, packed, out) {\n',
            '        if body && self.mod_player_geometry(m, who, alpha as f32, camera, packed, out) {\n            if chams {horizon_client::gfx::chams::mark_entity(&mut out[start..]);}\n')
        last=body.rfind('    }')
        body=body[:last]+'        if chams {horizon_client::gfx::chams::mark_entity(&mut out[start..]);}\n'+body[last:]
        return t[:start]+body+t[end:]
    edit('crates/horizon-client/src/game/perspective.rs',player)
    def layers(t):
        # These original types are outside ModuleChams.supportedRenderTypes.
        anchor='        if translucent_layer(m, layer) {\n            mark_oit(&mut out[start..]);\n        }\n'
        return once(t,anchor,anchor+'''        if crate::gfx::chams::active() && (layer.full_bright || layer.model == "breeze#wind") {
            for vertex in &mut out[start..] {vertex.layer|=crate::gfx::chams::UNSUPPORTED;}
        }
''')
    edit('crates/horizon-client/src/mobs/render.rs',layers)
    def renderer(t):
        t=once(t,'mod entity_outline;\n','mod entity_outline;\npub mod chams;\n')
        t=once(t,'    entity_outline: entity_outline::EntityOutline,\n','    entity_outline: entity_outline::EntityOutline,\n    chams: chams::ChamsRenderer,\n')
        t=once(t,'        let panorama = panorama::Panorama::new(&device, frame_format, panorama_faces);\n',
            '        let chams=chams::ChamsRenderer::new(&entity_shader,&entity_layout,frame_format,linear_light);\n        let panorama = panorama::Panorama::new(&device, frame_format, panorama_faces);\n')
        t=once(t,'            entity_outline,\n','            entity_outline,\n            chams,\n')
        t=once(t,'        let keep_depth = pack_reads_world\n','        let keep_depth = self.chams.pending() || pack_reads_world\n')
        t=once(t,'        if culling && let Some(cull) = &mut self.cull {\n            cull.bind_depth(&self.device, world_depth);\n',
            '''        let chams_size=self.world_post.size().filter(|_|scaled.is_some()).unwrap_or([self.config.width,self.config.height]);
        self.chams.prepare(&self.device,&self.queue,&mut self.uploads,&mut encoder,chams_size,world_depth,&self.entity_group,self.improved_transparency);
        if culling && let Some(cull) = &mut self.cull {
            cull.bind_depth(&self.device, world_depth);
''')
        t=once(t,'        let hand_in_world = (frame.hand.len() / 4) > 0\n',
            '        let hand_in_world = !self.chams.pending() && (frame.hand.len() / 4) > 0\n')
        t=once(t,'        if let Some((color, depth)) = &scaled\n            && pack_post\n',
            '        self.chams.composite(&mut encoder,world_view);\n        if let Some((color, depth)) = &scaled\n            && pack_post\n')
        return t
    edit('crates/horizon-client-render/src/gfx.rs',renderer)
    return {'hook_names':HOOK_NAMES,'helper_names':HELPER_NAMES}
