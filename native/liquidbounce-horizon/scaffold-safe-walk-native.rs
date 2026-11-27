// SPDX-License-Identifier: GPL-3.0-or-later
// Appended inside the existing edge_assistance module. Independent source mode
// instances retain independent centers, refreshable distances and tick counters.
thread_local! {static SCAFFOLD_STATE:RefCell<Option<State>>=const{RefCell::new(None)};}
pub(super) fn scaffold_input(game: &mut Game<'_>, settings: &Value, raw: [bool; 7]) -> [bool; 7] {
    let config = json!({"safeWalk":settings,"movementYaw":game.yaw});
    let epoch = world::current_epoch(game);
    world::with_scope(game, || {
        SCAFFOLD_STATE.with(|slot| {
            let mut slot = slot.borrow_mut();
            if slot.as_ref().is_none_or(|s| s.epoch != epoch) {
                *slot = Some(State::new(epoch, &config));
            }
            let state = slot.as_mut().expect("Scaffold SafeWalk");
            let mut output = keys(state.safe_walk(&config, input(raw), true));
            let down = settings["down"] == true;
            horizon_client::navigation_physics::set_scaffold_down(down);
            if down {
                output[5] = false;
            }
            output
        })
    })
}
