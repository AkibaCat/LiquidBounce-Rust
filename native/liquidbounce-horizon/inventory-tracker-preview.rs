// SPDX-License-Identifier: GPL-3.0-or-later
// Appended to perspective.rs: uses the native GUI entity renderer and full model.
impl Game<'_> {
    pub(super) fn client_inventory_tracker_preview(
        &self,
        batch: &mut HudBatch,
        model: &Mob,
        uuid: &str,
        name: &str,
        ox: f32,
        oy: f32,
    ) {
        let (x0, y0, x1, y1) = (ox + 26., oy + 8., ox + 75., oy + 78.);
        let (cx, cy) = ((x0 + x1) / 2., (y0 + y1) / 2.);
        let x_angle = ((cx - self.cursor.0) / 40.).atan();
        let y_angle = ((cy - self.cursor.1) / 40.).atan();
        let mut m = model.clone();
        m.extra.model_scale = 0.;
        m.body_yaw = 180. + x_angle * 20.;
        m.previous_body_yaw = m.body_yaw;
        m.head_yaw = m.body_yaw + x_angle * 20.;
        m.previous_head_yaw = m.head_yaw;
        m.head_pitch = -y_angle * 20.;
        m.previous_head_pitch = m.head_pitch;
        m.position.y += 0.125 * f64::from(u8::from(m.extra.crouching));
        m.previous = m.position;
        let camera = glam::DVec3::new(m.position.x, m.position.y, m.position.z);
        let atlas = &self.gui.atlas;
        let lookup = |name: &str| atlas.sprites.get(name).map(|r| [r.u0, r.v0, r.u1, r.v1]);
        let light = 240 | 240 << 8;
        let mut verts = Vec::new();
        let render = if !m.invisible || m.extra.spectator {
            horizon_client::mobs::render::render_with
        } else {
            horizon_client::mobs::render::render_layers
        };
        horizon_client::mobs::cem_anim::in_gui(|| {
            render(
                &m,
                1.,
                camera,
                &lookup,
                light,
                self.res.mob_data(),
                &self.worn_armor_layers(&m),
                &mut verts,
            );
        });
        let mut wings = 0..0;
        if horizon_client::mobs::render::layers_shown(&m) {
            let chest = m.extra.chest.as_deref();
            if !m.invisible {
                self.player_cape_geometry(&m, chest, camera, 1., light, &mut verts);
            }
            self.player_ears_geometry(&m, name, camera, 1., light, &mut verts);
            self.client_inventory_tracker_picture_layers(&m, uuid, camera, light, &mut verts);
            let start = verts.len();
            self.player_elytra_geometry(&m, chest, camera, 1., light, &mut verts);
            wings = start..verts.len();
            self.shoulder_parrots_geometry(&m, camera, 1., light, &mut verts);
            self.held_item_geometry(&m, camera, 1., light, false, None, &mut verts);
            self.head_item_geometry(&m, camera, 1., light, false, &mut verts);
        }
        let height = if m.extra.crouching { 1.5 } else { 1.8 };
        let gui = glam::Mat4::from_translation(glam::Vec3::new(cx, cy, 0.))
            * glam::Mat4::from_scale(glam::Vec3::new(30., 30., -30.))
            * glam::Mat4::from_translation(glam::Vec3::new(0., height / 2. + 0.0625, 0.))
            * glam::Mat4::from_rotation_z(std::f32::consts::PI)
            * glam::Mat4::from_rotation_x((y_angle * 20.).to_radians());
        let fire = (m.fire_ticks > 0 && !m.extra.spectator)
            .then_some((m.kind.width as f32, m.kind.height as f32));
        let glints = preview_glint_quads(&verts, gui, wings.clone(), self.glint_frame);
        let quads = player_preview_quads_no_cull(verts, gui, fire, &atlas.sprites, wings);
        emit_preview(batch, quads, glints, [x0, y0, x1, y1]);
    }
}
