#!/usr/bin/env python3
"""All CrystalView mixin phases; getY's beam consumers retain vanilla bobbing."""


def apply(edit, replace_once):
    def native(text):
        anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
        return replace_once(text,anchor,'''    /// The two EndCrystalRenderer and two EndCrystalModel parameters.
    pub(super) fn native_client_crystal_view(&self, size: &mut f32, y_translate: &mut f32,
        spin_speed: &mut f32, bounce: &mut f32) {
        if self.native_window() {
            mod_api::dispatch_client_crystal_view(self, mod_api::mods(), size, y_translate,
                spin_speed, bounce);
        }
    }

'''+anchor)
    edit('crates/horizon-client/src/game/native_hooks.rs',native)

    def model(text):
        text=replace_once(text,
            'pub fn poses(model: &Part, age: f32, show_bottom: bool) -> Poses {\n',
            '''pub fn poses(model: &Part, age: f32, show_bottom: bool) -> Poses {
    poses_with(model, age, show_bottom, 1.0, 1.0)
}
/// CrystalView changes the animationSpeed local and the getY return inside
/// setupAnim only. Other getY consumers (both crystal beams) stay vanilla.
pub fn poses_with(model: &Part, age: f32, show_bottom: bool,
    spin_speed: f32, bounce: f32) -> Poses {
''')
        text=replace_once(text,'    let speed = age * 3.0;\n','    let speed = age * 3.0 * spin_speed;\n')
        return replace_once(text,'    poses.get("outer_glass")[1] += y(age) * 16.0 / 2.0;\n',
            '    poses.get("outer_glass")[1] += y(age) * bounce * 16.0 / 2.0;\n')
    edit('crates/horizon-client/src/end_crystal.rs',model)

    def renderer(text):
        text=replace_once(text,'        for crystal in &self.crystals {\n',
            '''        let (mut size, mut y_translate, mut spin_speed, mut bounce) = (1.0, 0.0, 1.0, 1.0);
        self.native_client_crystal_view(&mut size, &mut y_translate, &mut spin_speed, &mut bounce);
        for crystal in &self.crystals {
''')
        text=replace_once(text,'            let poses = horizon_client::end_crystal::poses(model, age, crystal.show_bottom);\n',
            '            let poses = horizon_client::end_crystal::poses_with(model, age, crystal.show_bottom, spin_speed, bounce);\n')
        return replace_once(text,'''            let base = glam::Mat4::from_translation(rel.as_vec3())
                * glam::Mat4::from_scale(glam::Vec3::splat(2.0))
''','''            let base = glam::Mat4::from_translation(rel.as_vec3())
                * glam::Mat4::from_translation(glam::Vec3::new(0.0, y_translate, 0.0))
                * glam::Mat4::from_scale(glam::Vec3::splat(2.0 * size))
''')
    edit('crates/horizon-client/src/game/end.rs',renderer)
    return {'hook_names':['client_crystal_view'],'helper_names':[]}
