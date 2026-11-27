// SPDX-License-Identifier: GPL-3.0-or-later
//! Crosshair's original landing-tick API over the shared full replay.
use super::Game;
pub(super) fn collision(
    game: &mut Game<'_>,
    ticks: i32,
    add_y: f64,
) -> std::result::Result<Option<i32>, String> {
    let mut landing = None;
    super::falling_forecast::visit(game, ticks, add_y, |player, tick| {
        if player.ground.on_ground {
            landing = Some(tick);
            true
        } else {
            false
        }
    })?;
    Ok(landing)
}
