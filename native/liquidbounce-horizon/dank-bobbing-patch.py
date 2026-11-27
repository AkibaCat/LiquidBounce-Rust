#!/usr/bin/env python3
"""DankBobbing at the real shared world/hand bob matrix consumer.

Apply after the main generator's NoBob/NoHurtCam fov() transform. The camera
entity's actual AvatarBob supplies the player inputs (including spectators).
"""
HOOK_NAMES = ['client_dank_bobbing']
HOOKS = '''    /// GameRenderer.bobView custom branch; existing HEAD cancellation wins.
    pub(super) fn native_client_dank_bobbing(&self, enabled: &mut bool, motion: &mut f32) {
        if self.native_window() {
            mod_api::dispatch_client_dank_bobbing(self, mod_api::mods(), enabled, motion);
        }
    }

'''
DANK = '''/// ModuleDankBobbing's complete bobView replacement. Translations use the
/// walked distance; the two rotations deliberately use bob itself in the source.
pub fn bob_view_dank(state: &BobState, motion: f32, pose: &mut Mat4) {
    let Some((walk, bob)) = state.player else { return; };
    pose.translate(
        trig::sin(f64::from(walk * PI)) * bob * 0.5,
        -(trig::cos(f64::from(walk * PI)) * bob).abs(),
        0.0,
    );
    let z = trig::sin(f64::from(bob * PI)) * bob * (3.0 + motion);
    let x = (trig::cos(f64::from(bob * PI - (0.2 + motion))) * bob).abs() * 5.0;
    // The source calls PoseStack.rotate(Axis.rotationDegrees(...)), whose
    // quaternion matrix path differs in float rounding from rotateDegrees.
    pose.rotate_quaternion(Quat::rotation_z(z * DEG_TO_RAD));
    pose.rotate_quaternion(Quat::rotation_x(x * DEG_TO_RAD));
}
pub fn bob_matrix_dank(state: &BobState, damage_tilt_strength: f64,
    bob_view_option: bool, motion: Option<f32>) -> Mat4 {
    let mut pose = Mat4::IDENTITY;
    bob_hurt(state, damage_tilt_strength, &mut pose);
    if bob_view_option {
        if let Some(motion) = motion { bob_view_dank(state, motion, &mut pose); }
        else { bob_view(state, &mut pose); }
    }
    pose
}

'''
PLAYER_BOB = '''    /// CameraRenderState.entityRenderState.isPlayer: mannequins share Avatar
    /// animation storage but are not Player. Spectated players own their history.
    pub(super) fn camera_player_bob(&self, partial: f32) -> Option<(f32, f32)> {
        if let Some(mob) = self.camera_mob() {
            return (&*mob.kind.id == "minecraft:player").then(|| (
                mob.extra.avatar.backwards_interpolated_walk_distance(partial),
                mob.extra.avatar.interpolated_bob(partial)));
        }
        if self.camera_entity().is_some() { return None; }
        self.player.as_ref().map(|_| (
            self.avatar.backwards_interpolated_walk_distance(partial),
            self.avatar.interpolated_bob(partial)))
    }

'''

def apply(edit, once):
    edit('crates/horizon-client/src/game/native_hooks.rs', lambda t: once(t,
        '    /// Independent GameRenderer.bobHurt / bobView HEAD cancellations.\n',
        HOOKS + '    /// Independent GameRenderer.bobHurt / bobView HEAD cancellations.\n'))
    def camera(text):
        text = once(text, '        /// `Matrix4f.rotation(Quaternionfc)`.\n', '''        /// Matrix4f.rotate(Quaternionfc), with identity/translation shortcuts.
        /// Camera bob poses are always affine; preserve JOML operation order.
        pub fn rotate_quaternion(&mut self, q: Quat) {
            if self.properties & IDENTITY != 0 { self.rotation(q); }
            else if self.properties & TRANSLATION != 0 {
                let (x, y, z) = (self.get(3, 0), self.get(3, 1), self.get(3, 2));
                self.rotation(q);
                self.set_translation(x, y, z);
            } else { self.rotate_affine(q); }
        }
        /// `Matrix4f.rotation(Quaternionfc)`.
''')
        return once(text,
            '/// The bob matrix `GameRenderer.renderLevel` multiplies into the projection\n',
            DANK + '/// The bob matrix `GameRenderer.renderLevel` multiplies into the projection\n')
    edit('crates/horizon-client-render/src/camera.rs', camera)
    def fov(text):
        text = once(text, '    pub(super) fn bob_matrix(&self, partial: f32)',
            PLAYER_BOB + '    pub(super) fn bob_matrix(&self, partial: f32)')
        text = once(text, '''            player: Some((
                self.avatar.backwards_interpolated_walk_distance(partial),
                self.avatar.interpolated_bob(partial),
            )),''', '''            player: self.camera_player_bob(partial),''')
        text = once(text, '        let applied = bob_matrix(&filtered, tilt, view_enabled).m;\n', '''        let mut dank = false;
        let mut motion = 5.0;
        self.native_client_dank_bobbing(&mut dank, &mut motion);
        let applied = horizon_client::camera::bob_matrix_dank(&filtered, tilt, view_enabled,
            dank.then_some(motion)).m;
''')
        return once(text, '        let (walk, bob) = state.player.expect("local player\'s camera has bob history");\n',
            '        let (walk, bob) = state.player.unwrap_or((0.0, 0.0));\n')
    edit('crates/horizon-client/src/game/fov.rs', fov)
    return {'hook_names': HOOK_NAMES, 'helper_names': []}
