#!/usr/bin/env python3
"""Both original NoPose mixins: local setPose and global Avatar dimensions."""

HOOK_NAMES = ['client_no_pose_policy']

HOOKS = '''    /// Capture the local-only pose cancel and the global Avatar dimension choice.
    pub(super) fn native_client_no_pose_policy(&self, no_swim: &mut bool, sneak_height: &mut u8) {
        if !self.native_window() { return; }
        mod_api::dispatch_client_no_pose_policy(self, mod_api::mods(), no_swim, sneak_height);
    }
    pub(super) fn sync_native_no_pose(&mut self) {
        if !self.native_window() { return; }
        let (mut no_swim, mut sneak_height) = (false, 0);
        self.native_client_no_pose_policy(&mut no_swim, &mut sneak_height);
        // MixinAvatar has no side or identity guard: remote avatars, mannequins,
        // and integrated-server copies receive this getDefaultDimensions result.
        horizon_client::no_pose::publish(sneak_height);
        if let Some(player) = self.player.as_mut() {
            player.client_no_swim_pose = no_swim;
        }
        // Do not refresh a cached box on toggle: the original module does not.
    }

'''


def policy_source():
    return '''// SPDX-License-Identifier: GPL-3.0-or-later
//! NoPose's global Avatar.getDefaultDimensions(CROUCHING) return value.
use std::sync::atomic::{AtomicU8, Ordering};
static SNEAK_HEIGHT: AtomicU8 = AtomicU8::new(0);
pub fn publish(choice: u8) { SNEAK_HEIGHT.store(choice, Ordering::Release); }
pub fn crouching_dimensions(scale: f32) -> Option<(f32, f32, f32, [f64; 2])> {
    let (height, eye) = match SNEAK_HEIGHT.load(Ordering::Acquire) {
        1 => (1.8, 1.54), 2 => (1.65, 1.54), 3 => (1.5, 1.27), _ => return None,
    };
    // EntityAttachments.scale widens the original float before its double
    // multiplication. This differs from the rounded scaled box height.
    Some((0.6 * scale, height * scale, eye * scale, [f64::from(height) * f64::from(scale), 0.6 * f64::from(scale)]))
}
'''


def apply(edit, replace_once):
    edit('crates/horizon-client/src/no_pose.rs', lambda _: policy_source())
    edit('crates/horizon-client/src/client_lib.rs', lambda text: replace_once(text, 'pub mod pose;\n', 'pub mod pose;\npub mod no_pose;\n'))
    edit('crates/horizon-client/src/game/native_hooks.rs', lambda text: replace_once(text,
        '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',
        HOOKS + '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))

    def game(text):
        for anchor in ['    fn tick(&mut self) -> Result<()> {\n', '    fn render_frame(&mut self) -> Result<bool> {\n']:
            text = replace_once(text, anchor, anchor + '        self.sync_native_no_pose();\n')
        return text
    edit('crates/horizon-client/src/game.rs', game)
    edit('crates/horizon-client/src/motion.rs', lambda text: replace_once(text,
        '    pub pose: crate::pose::Pose,\n', '    pub pose: crate::pose::Pose,\n    /// Only Game.player receives this Entity.setPose cancellation snapshot.\n    pub client_no_swim_pose: bool,\n'))
    edit('crates/horizon-client/src/motion/body.rs', lambda text: replace_once(text,
        '            pose: crate::pose::Pose::Standing,\n', '            pose: crate::pose::Pose::Standing,\n            client_no_swim_pose: false,\n'))

    def pose(text):
        text = replace_once(text, '    pub fn dimensions(self, scale: f32) -> Dimensions {\n',
            '''    pub fn dimensions(self, scale: f32) -> Dimensions {
        let no_pose = if self == Self::Crouching { crate::no_pose::crouching_dimensions(scale) } else { None };
''')
        text = replace_once(text, '            Self::Crouching => (0.6 * scale, 1.5 * scale, 1.27 * scale),\n',
            '''            Self::Crouching => no_pose.map(|d| (d.0, d.1, d.2))
                .unwrap_or((0.6 * scale, 1.5 * scale, 1.27 * scale)),
''')
        text = replace_once(text, '            eye_height,\n', '            eye_height,\n            no_pose_attachments: no_pose.map(|d| d.3),\n')
        text = replace_once(text, '    pub eye_height: f32,\n', '    pub eye_height: f32,\n    /// Cached with this box, exactly like the original EntityDimensions attachments.\n    pub no_pose_attachments: Option<[f64; 2]>,\n')
        text = replace_once(text, 'impl PlayerMotion {\n', '''impl PlayerMotion {
    pub fn passenger_attachment_y(&self) -> f64 {
        self.dimensions.no_pose_attachments.map(|a| a[0]).unwrap_or(f64::from(self.dimensions.height))
    }
''')
        return replace_once(text, '    pub fn set_pose(&mut self, pose: Pose) -> Result<()> {\n',
            '''    pub fn set_pose(&mut self, pose: Pose) -> Result<()> {
        // MixinEntity.setPose HEAD: cancel only the actual local player, even
        // for an equal pose; no mutation to Entity's separate swimming flag.
        if self.client_no_swim_pose && self.client_side && !self.server_copy
            && !self.replaying && pose == Pose::Swimming {
            return Ok(());
        }
''')
    edit('crates/horizon-client/src/pose.rs', pose)
    edit('crates/horizon-client/src/game/unloaded_chunk_tests.rs', lambda text: replace_once(text,
        '    let body = Dimensions {\n', '    let body = Dimensions {\n        no_pose_attachments: None,\n'))

    def passengers(text):
        original = 'f64::from(p.dimensions.height)'
        if text.count(original) != 3:
            raise ValueError('Expected all three local Avatar passenger attachments')
        return text.replace(original, 'p.passenger_attachment_y()')
    edit('crates/horizon-client/src/game/passengers.rs', passengers)
    edit('crates/horizon-client/src/mobs/riders.rs', lambda text: replace_once(text,
        'pub fn passenger_attachment(v: &Mob) -> [f64; 3] {\n',
        '''pub fn passenger_attachment(v: &Mob) -> [f64; 3] {
    if avatar(v) && let Some(a) = v.extra.avatar_pose.dimensions(v.model_scale()).no_pose_attachments {
        return [0.0, a[0], 0.0];
    }
'''))
    edit('crates/horizon-client/src/mobs/riders.rs', lambda text: replace_once(text,
        'pub fn vehicle_attachment(r: &Mob, vehicle_width: f32) -> f64 {\n',
        '''pub fn vehicle_attachment(r: &Mob, vehicle_width: f32) -> f64 {
    if avatar(r) && let Some(a) = r.extra.avatar_pose.dimensions(r.model_scale()).no_pose_attachments {
        return a[1];
    }
'''))
    edit('crates/horizon-client/src/game/mounts.rs', lambda text: replace_once(text,
        '                f64::from(0.6_f32),\n',
        '''                self.player.as_ref().and_then(|p| p.dimensions.no_pose_attachments)
                    .map_or(f64::from(0.6_f32), |a| a[1]),
'''))
    edit('crates/horizon-client/src/game/remote_vehicles.rs', lambda text: replace_once(text,
        '            Some(Vec3::new(at.x, at.y - f64::from(0.6_f32 * scale), at.z))\n',
        '''            let offset = self.player.as_ref().and_then(|p| p.dimensions.no_pose_attachments)
                .map_or(f64::from(0.6_f32 * scale), |a| a[1]);
            Some(Vec3::new(at.x, at.y - offset, at.z))
'''))
    return {'hook_names': HOOK_NAMES, 'helper_names': []}
