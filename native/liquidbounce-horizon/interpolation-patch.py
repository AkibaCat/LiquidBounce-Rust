#!/usr/bin/env python3
"""Client-only Abstract/Stepped/Linear interpolation step query seams."""


def apply(edit, replace_once):
    def native(text):
        anchor = '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
        return replace_once(text, anchor, '''    /// A snapshot before client entity borrows. No server interpolation is changed.
    pub(super) fn native_client_interpolation_reduction(&self) -> Option<i32> {
        let (mut enabled, mut reduce_by) = (false, 3);
        self.native_client_interpolation_policy(&mut enabled, &mut reduce_by);
        enabled.then_some(reduce_by)
    }
    pub(super) fn native_client_interpolation_policy(&self, enabled: &mut bool, reduce_by: &mut i32) {
        if self.native_window() {
            mod_api::dispatch_client_interpolation_policy(self, mod_api::mods(), enabled, reduce_by);
        }
    }

''' + anchor)

    def stepped(text):
        text = replace_once(text, '    steps: i32,\n', '    steps: i32,\n    client_step_reduction: Option<i32>,\n')
        text = replace_once(text, '            steps,\n', '            steps,\n            client_step_reduction: None,\n')
        text = replace_once(text, '    /// `hasActiveInterpolation`.\n', '''    /// Applied by the local client before starting an interpolation. The type's
    /// update interval stays intact, including InterpolationData.advance's divisor.
    pub fn set_client_step_reduction(&mut self, reduction: Option<i32>) {
        self.client_step_reduction = reduction;
    }
    fn start_steps(&self) -> i32 {
        self.client_step_reduction.map_or(self.steps, |n| self.steps.wrapping_sub(n).max(0))
    }
    /// `hasActiveInterpolation`.
''')
        text = replace_once(text, '        if self.steps == 0 {\n', '        if self.start_steps() == 0 {\n')
        original = '                self.steps,\n'
        if text.count(original) != 2:
            raise ValueError('Expected the two startInterpolating step reads')
        return text.replace(original, '                self.start_steps(),\n')

    def remote(text):
        text = replace_once(text, '''        let dragon = self.dragon.as_ref().is_some_and(|d| d.id as i32 == id);
''', '''        let step_reduction = self.native_client_interpolation_reduction();
        let steps = |original: i32| step_reduction.map_or(original, |n| original.wrapping_sub(n).max(0));
        let linear_steps = steps(3) as u32;
        let driven_boat = self.riding.filter(|&boat| self.controls_boat(boat));
        let dragon = self.dragon.as_ref().is_some_and(|d| d.id as i32 == id);
''')
        original = '                if let Some(at) = stepped_move(handler, packet, entity, position, path, rotation) {\n'
        if text.count(original) != 2:
            raise ValueError('Expected mob and armor stand interpolation scheduling')
        text = text.replace(original, '                handler.set_client_step_reduction(step_reduction);\n' + original)
        original = '                    h.move_to(position, on_ground);\n'
        if text.count(original) != 2:
            raise ValueError('Expected local and remote fishing hook interpolation')
        text = text.replace(original, '''                    if linear_steps == 3 { h.move_to(position, on_ground); }
                    else { h.move_to_with_steps(position, on_ground, linear_steps); }
''')
        text = replace_once(text, '''                o.on_ground = on_ground;
                r.lerps.insert(
''', '''                o.on_ground = on_ground;
                if linear_steps == 0 {
                    o.position = at;
                    o.previous = at;
                    r.lerps.remove(&id);
                    return;
                }
                r.lerps.insert(
''')
        original = '                        steps: 3,\n'
        if text.count(original) != 2:
            raise ValueError('Expected orb and base linear step scheduling')
        text = text.replace(original, '                        steps: linear_steps,\n')
        text = replace_once(text, '''                let rotation = rotation.or_else(|| r.lerps.get(&id).and_then(|l| l.rotation));
                r.lerps.insert(
''', '''                let rotation = rotation.or_else(|| r.lerps.get(&id).and_then(|l| l.rotation));
                if linear_steps == 0 {
                    r.lerps.remove(&id);
                    if let Some(b) = self.boats.iter_mut().find(|b| b.id == id as u32)
                        && Some(b.id) != driven_boat {
                        b.position = at;
                        b.previous = at;
                        if let Some((yaw, _)) = rotation { b.yaw = yaw; b.previous_yaw = yaw; }
                        b.on_ground = on_ground;
                    }
                    if let Some(c) = self.carts.iter_mut().find(|c| c.id == id as u32) {
                        c.position = at;
                        c.previous = at;
                        if let Some((yaw, pitch)) = rotation {
                            c.yaw = yaw; c.previous_yaw = yaw;
                            c.pitch = pitch; c.previous_pitch = pitch;
                        }
                    }
                    if let Some(d) = self.dragon.as_mut().filter(|d| d.id as i32 == id) {
                        d.position = at;
                        d.previous = at;
                        if let Some((yaw, _)) = rotation { d.yaw = yaw; d.previous_yaw = yaw; }
                    }
                } else {
                r.lerps.insert(
''')
        return replace_once(text, '''                        steps: linear_steps,
                    },
                );
            }
        }
        self.turn_remote_object''', '''                        steps: linear_steps,
                    },
                );
                }
            }
        }
        self.turn_remote_object''')

    def fishing(text):
        text = replace_once(text, '''    fn start(&mut self, target: [f64; 3]) {
        let target''', '''    fn start(&mut self, target: [f64; 3]) { self.start_with_steps(target, 3); }
    fn start_with_steps(&mut self, target: [f64; 3], steps: u32) {
        let target''')
        text = replace_once(text, '            self.steps = 3;\n', '            self.steps = steps;\n')
        original = '''    pub(super) fn move_to(&mut self, target: [f64; 3], on_ground: bool) {
        self.on_ground = resting(self.on_ground, self.position, target, on_ground);
        self.lerp.start(target);
    }'''
        if text.count(original) != 2:
            raise ValueError('Expected both complete fishing hook move_to methods')
        return text.replace(original, '''    pub(super) fn move_to(&mut self, target: [f64; 3], on_ground: bool) {
        self.move_to_with_steps(target, on_ground, 3);
    }
    pub(super) fn move_to_with_steps(&mut self, target: [f64; 3], on_ground: bool, steps: u32) {
        self.on_ground = resting(self.on_ground, self.position, target, on_ground);
        if steps == 0 {
            self.lerp.steps = 0;
            self.position = Vec3::new(target[0], target[1], target[2]);
            self.previous = self.position;
        } else {
            if steps == 3 { self.lerp.start(target); }
            else { self.lerp.start_with_steps(target, steps); }
        }
    }''')

    def displays(text):
        text = replace_once(text, '''    fn move_to(&mut self, position: [f64; 3], rotation: Option<(f32, f32)>) {
''', '''    fn move_to(&mut self, position: [f64; 3], rotation: Option<(f32, f32)>) {
        self.move_to_with_reduction(position, rotation, None);
    }
    fn move_to_with_reduction(&mut self, position: [f64; 3], rotation: Option<(f32, f32)>, reduction: Option<i32>) {
''')
        text = replace_once(text, '        self.steps = self.data.teleport_duration.clamp(0, 59);\n', '''        let original = self.data.teleport_duration.clamp(0, 59);
        self.steps = reduction.map_or(original, |n| original.wrapping_sub(n).max(0));
''')
        return replace_once(text, '''        if let Some(d) = self.text_displays.get_mut(&id) {
            d.move_to(position, rotation);
''', '''        let reduction = self.native_client_interpolation_reduction();
        if let Some(d) = self.text_displays.get_mut(&id) {
            if reduction.is_none() { d.move_to(position, rotation); }
            else { d.move_to_with_reduction(position, rotation, reduction); }
''')

    edit('crates/horizon-client/src/game/native_hooks.rs', native)
    edit('crates/horizon-world/src/stepped_interpolation.rs', stepped)
    edit('crates/horizon-client/src/game/remote.rs', remote)
    edit('crates/horizon-client/src/game/fishing.rs', fishing)
    edit('crates/horizon-client/src/game/text_displays.rs', displays)
    return {'hook_names': ['client_interpolation_policy'], 'helper_names': []}
