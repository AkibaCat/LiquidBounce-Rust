// SPDX-License-Identifier: GPL-3.0-or-later
// Appended to player_hooks.rs so a remote/saved avatar retains its own UUID.
impl Game<'_> {
    pub(super) fn client_inventory_tracker_picture_layers(
        &self,
        m: &Mob,
        uuid: &str,
        camera: glam::DVec3,
        light: u32,
        out: &mut Vec<Vertex>,
    ) {
        if !engine::any() {
            return;
        }
        let avatar = model(m, Kind::Player, u128::from_str_radix(uuid, 16).ok());
        let mut draw = DrawPlayer {
            game: self,
            m,
            pass: Pass::Picture,
            base: render::base_transform(m, 1., camera),
            camera,
            alpha: 1.,
            light,
            gpu: false,
            out,
            bounds: None,
            body: None,
            poses: Poses::pooled(),
        };
        engine::player_layers(&avatar, &mut draw);
    }
}
