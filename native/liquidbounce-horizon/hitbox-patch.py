#!/usr/bin/env python3
"""ModuleHitbox's complete pick, component and F3+B consumers; after Reach."""
from pathlib import Path

HERE=Path(__file__).resolve().parent
HOOK_NAMES=['client_entity_margin','client_debug_hitbox_margin','client_hitbox_component_margin']
HELPER_NAMES=['client_targeting_entity']
HOOKS=(HERE/'hitbox-host-methods.rs').read_text(encoding='utf-8')


def apply(edit,replace_once):
    def hooks(text):
        text=replace_once(text,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',HOOKS+'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n')
        # AttackRange.isInRange's field read: local client receiver only.
        return replace_once(text,'        let margin = value("hitbox_margin", 0.3);\n','        let mut margin = value("hitbox_margin", 0.3);\n        self.native_client_hitbox_component_margin(&mut margin);\n')
    edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
    def game(text):
        return replace_once(text,'            if let Some(t) = ray_box(eye, dir, reach, m.bounds())\n',
            '            let margin=self.client_pick_margin(m.id as i32,0.0);\n            if let Some(t) = ray_box(eye, dir, reach, m.bounds().inflate(Vec3::new(margin,margin,margin)))\n')
    edit('crates/horizon-client/src/game.rs',game)
    def stand(text):
        start=text.index('fn armor_stand_hit(')
        before=text[:start];body=text[start:]
        body=replace_once(body,'            let b = a.bounds();\n','            let margin=self.client_pick_margin(a.id as i32,0.0);\n            let b = a.bounds().inflate(Vec3::new(margin,margin,margin));\n')
        return before+body
    edit('crates/horizon-client/src/game/armor_stands.rs',stand)
    def reach(text):
        # Reach's canonical EntityHitResult must use the same getPickRadius
        # override. Dragon parts are nonliving, so their original 0 survives.
        return replace_once(text,'                kind, index, id: id as i32, local_id, position, bounds, pick_radius,\n',
            '                kind, index, id: id as i32, local_id, position, bounds,\n                pick_radius:self.client_pick_margin(id as i32,pick_radius as f32),\n')
    edit('crates/horizon-client/src/game/native_hooks.rs',reach)
    def hand(text):
        text=replace_once(text,'        let reach = attack_reach(range, self.creative);\n',
            '        let mut reach = attack_reach(range, self.creative);\n        self.native_client_hitbox_component_margin(&mut reach.2);\n')
        return replace_once(text,'            let reach = attack_reach(r, self.creative);\n',
            '            let mut reach = attack_reach(r, self.creative);\n            self.native_client_hitbox_component_margin(&mut reach.2);\n')
    edit('crates/horizon-client/src/game/hand.rs',hand)
    def gizmos(text):
        anchor='    /// The same for an entity whose box is not centred on its position\n'
        method='''    /// Hitbox mixin expands the actual AABB before the original renderer.
    /// The red eye plane uses the expanded minY, while the view arrow keeps
    /// the entity's original interpolated feet, just like showHitboxes.
    pub fn hitbox_with_margin(
        &mut self, position:[f64;3], size:(f32,f32), eye_height:f32,
        living:bool, view:(f32,f32), margin:f32,
    ) {
        let half=f64::from(size.0)/2.0;
        let amount=f64::from(margin);
        let min=[position[0]-half-amount,position[1]-amount,position[2]-half-amount];
        let max=[position[0]+half+amount,position[1]+f64::from(size.1)+amount,position[2]+half+amount];
        self.entity_box(min,max,position,eye_height,living,view);
    }
'''
        return replace_once(text,anchor,method+anchor)
    edit('crates/horizon-client/src/gizmos.rs',gizmos)
    def debug(text):
        text=replace_once(text,'''            g.hitbox(
                pos,
                (m.width(), m.height()),
                m.eye_height() as f32,
                true,
                view,
            );''','''            g.hitbox_with_margin(
                pos,
                (m.width(), m.height()),
                m.eye_height() as f32,
                true,
                view,
                self.client_debug_hitbox_margin(m.id as i32),
            );''')
        text=replace_once(text,'''            g.hitbox(
                at(a.previous, a.position),
                size,
                eye,
                true,
                (0.0, a.render_yaw(alpha)),
            );''','''            g.hitbox_with_margin(
                at(a.previous, a.position),
                size,
                eye,
                true,
                (0.0, a.render_yaw(alpha)),
                self.client_debug_hitbox_margin(a.id as i32),
            );''')
        text=replace_once(text,'''            g.hitbox(
                at(d.previous, d.position),
                (16.0, 8.0),
                6.8,
                true,
                (0.0, d.yaw),
            );''','''            g.hitbox_with_margin(
                at(d.previous, d.position),
                (16.0, 8.0),
                6.8,
                true,
                (0.0, d.yaw),
                self.client_debug_hitbox_margin(self.client_targeting_dragon_id()),
            );''')
        text=replace_once(text,'''            if (super::dragon::SPECIAL..super::dragon::SPECIAL + 8).contains(&id) {
                g.cuboid(''','''            if (super::dragon::SPECIAL..super::dragon::SPECIAL + 8).contains(&id) {
                // The mixin's entity local is the parent EnderDragon, even
                // while the returned expression is a part's bounding box.
                let amount=f64::from(self.client_debug_hitbox_margin(self.client_targeting_dragon_id()));
                let b=b.inflate(Vec3::new(amount,amount,amount));
                g.cuboid(''')
        return text
    edit('crates/horizon-client/src/game/debug_keys.rs',debug)
    return {'hook_names':HOOK_NAMES,'helper_names':HELPER_NAMES}
