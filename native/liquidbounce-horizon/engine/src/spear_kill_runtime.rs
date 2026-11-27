// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{Host, WorldPrimitivesEvent};
use horizon_client::world_primitives::PrimitiveShape;
use liquidbounce_behaviors::spear_kill::{Host as SpearHost, Kinetic, SpearKill, Target, Vector};
struct Adapter<'a> {
    hooks: &'a LiquidBounceHooks,
    host: &'a mut dyn Host,
}
impl SpearHost for Adapter<'_> {
    fn with_spear_kill<R>(&mut self, operation: impl FnOnce(&mut SpearKill) -> R) -> R {
        let state = self.hooks.state();
        let mut modules = state.behaviors.borrow_mut();
        operation(&mut modules.spear_kill)
    }
    fn using_spear(&mut self) -> bool {
        self.host
            .client_spear_kill_context()
            .is_some_and(|v| v[0] != 0.0)
    }
    fn kinetic(&mut self) -> Option<Kinetic> {
        let v = self.host.client_spear_kill_context()?;
        (v[1] != 0.0).then_some(Kinetic {
            delay: v[2] as i32,
            damage_duration: v[3] as i32,
        })
    }
    fn ticks_using(&mut self) -> i32 {
        self.host
            .client_spear_kill_context()
            .map_or(0, |v| v[4] as i32)
    }
    fn use_down(&mut self) -> bool {
        self.host
            .client_spear_kill_context()
            .is_some_and(|v| v[5] != 0.0)
    }
    fn attack_down(&mut self) -> bool {
        self.host
            .client_spear_kill_context()
            .is_some_and(|v| v[6] != 0.0)
    }
    fn restart_using(&mut self) {
        self.host.client_spear_kill_restart();
    }
    fn target(&mut self, maximum: f32) -> Option<Target> {
        let [id, distance] = self.host.client_spear_kill_target(maximum)?;
        Some(Target {
            id: id as i32,
            distance,
        })
    }
    fn predict(&mut self, id: i32, ticks: i32) -> Option<Vector> {
        self.host.client_spear_kill_predict(id, ticks)
    }
    fn eye(&mut self) -> Vector {
        self.host.client_spear_kill_eye().unwrap_or([0.0; 3])
    }
    fn look(&mut self) -> Vector {
        self.host.client_spear_kill_look().unwrap_or([0.0; 3])
    }
    fn velocity(&mut self, value: Vector) {
        self.host.client_spear_kill_velocity(value);
    }
}
impl LiquidBounceHooks {
    pub(crate) fn spear_kill_tick(&self, host: &mut dyn Host) {
        let running = {
            let state = self.state();
            let modules = state.behaviors.borrow();
            modules.query_running(&state.registry, "SpearKill")
        };
        SpearKill::tick(running, &mut Adapter { hooks: self, host });
    }
    pub(crate) fn spear_kill_render(&self, host: &dyn Host, event: &mut WorldPrimitivesEvent<'_>) {
        let (target, settings) = {
            let state = self.state();
            let modules = state.behaviors.borrow();
            if !modules.query_running(&state.registry, "SpearKill")
                || !modules.spear_kill.settings.preview
            {
                return;
            }
            (
                modules.spear_kill.preview_target,
                modules.spear_kill.settings,
            )
        };
        let Some(target) = target else {
            return;
        };
        let Ok(boxes) =
            serde_json::from_str::<Vec<[f64; 6]>>(&host.client_spear_kill_boxes(target))
        else {
            return;
        };
        for b in boxes {
            if settings.fill_color as u32 >> 24 != 0 {
                event
                    .primitives
                    .push(PrimitiveShape::FilledBox {
                        min: [b[0], b[1], b[2]],
                        max: [b[3], b[4], b[5]],
                        color: settings.fill_color as u32,
                    })
                    .through_walls();
            }
            if settings.outline_color as u32 >> 24 != 0 {
                event
                    .primitives
                    .push(PrimitiveShape::BoxOutline {
                        min: [b[0], b[1], b[2]],
                        max: [b[3], b[4], b[5]],
                        color: settings.outline_color as u32,
                        width: 1.0,
                    })
                    .through_walls();
            }
        }
    }
}
