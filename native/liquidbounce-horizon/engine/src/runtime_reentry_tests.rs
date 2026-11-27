// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use horizon_client::mod_api::{self, ClientUseKeyPressedEvent};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

struct InputHooks(Arc<LiquidBounceHooks>);

impl Hooks for InputHooks {
    fn client_directional_input(&self, host: &mut dyn Host, event: &mut ClientDirectionalInputEvent) {
        self.0.client_directional_input(host, event);
    }

    fn client_use_key_pressed(&self, host: &dyn Host, event: &mut ClientUseKeyPressedEvent) {
        self.0.client_use_key_pressed(host, event);
    }
}

struct ReentrantInputHost {
    hooks: Arc<LiquidBounceHooks>,
    use_key_calls: AtomicUsize,
}

impl Host for ReentrantInputHost {
    fn client_fast_use_key(&self) -> bool {
        // Fail immediately on a regression instead of blocking the test runner.
        assert!(
            self.hooks.state.try_lock().is_ok(),
            "use-key sampling must release actor state before native callbacks"
        );
        let mut pressed = true;
        let hooks: Vec<Box<dyn Hooks>> = vec![Box::new(InputHooks(self.hooks.clone()))];
        mod_api::dispatch_client_use_key_pressed(self, &hooks, &mut pressed);
        self.use_key_calls.fetch_add(1, Ordering::Relaxed);
        pressed
    }

    fn client_inventory_move_keys(&mut self) -> [bool; 7] {
        [false; 7]
    }
    fn client_sneak_context(&self) -> Option<[bool; 2]> {
        None
    }
    fn client_no_slow_context(&self) -> Option<[f64; 15]> {
        None
    }
    fn client_target_strafe_query(&mut self, _input: &str) -> String {
        "null".into()
    }
    fn client_edge_assistance_result(&self) -> Option<String> {
        None
    }
    fn client_free_cam_navigation_result(&self) -> Option<String> {
        None
    }
    fn client_auto_walk_result(&self) -> Option<String> {
        None
    }
    fn client_crosshair_clicker_context(&self) -> Option<serde_json::Value> {
        None
    }
    fn client_criticals_context(&self) -> Option<serde_json::Value> {
        None
    }
    fn client_player_rotation(&self) -> [f32; 2] {
        [0.; 2]
    }
    fn client_player_input_tick_count(&self) -> Option<i32> {
        None
    }
    fn client_anti_hunger_swimming(&self) -> Option<bool> {
        Some(false)
    }
    fn client_vehicle_control_state(&mut self, _target: i32) -> Option<serde_json::Value> {
        None
    }
    fn client_scaffold_features(&mut self, _input: &str) -> String {
        "null".into()
    }
    fn client_vanilla_sin_cos(&self, radians: f64) -> [f32; 2] {
        let (sin, cos) = radians.sin_cos();
        [sin as f32, cos as f32]
    }
}

#[test]
fn directional_input_use_key_callback_can_reenter_without_actor_lock() {
    // Build the actor without installing process-wide packet or text callbacks.
    let state = State::default();
    let (packets, sword_block, kill_aura_blocking) = {
        let modules = state.behaviors.borrow();
        (
            Arc::new(crate::packets::PacketPipeline::with_no_swing(
                modules.no_swing.clone(),
            )),
            modules.sword_block.clone(),
            modules.kill_aura.blocking.clone(),
        )
    };
    let hooks = Arc::new(LiquidBounceHooks {
        state: Mutex::new(state),
        _packets: packets,
        sword_block,
        kill_aura_blocking,
        started: Instant::now(),
        #[cfg(feature = "webview")]
        browser: None,
    });
    let mut host = ReentrantInputHost {
        hooks: hooks.clone(),
        use_key_calls: AtomicUsize::new(0),
    };
    let hooks: Vec<Box<dyn Hooks>> = vec![Box::new(InputHooks(hooks))];
    let (mut forward, mut backward, mut left, mut right, mut jump, mut shift, mut sprint) =
        (true, false, false, false, false, false, false);
    mod_api::dispatch_client_directional_input(
        &mut host, &hooks, &mut forward, &mut backward, &mut left, &mut right,
        &mut jump, &mut shift, &mut sprint,
    );

    assert_eq!(host.use_key_calls.load(Ordering::Relaxed), 1);
    assert!(forward, "disabled FastUse must preserve movement");
}
