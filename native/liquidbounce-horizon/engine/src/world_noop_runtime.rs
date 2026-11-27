// SPDX-License-Identifier: GPL-3.0-or-later
use crate::behaviors::BehaviorModules;
use liquidbounce_core::ModuleRegistry;
pub(crate) fn tick_world_noops(modules: &mut BehaviorModules, registry: &ModuleRegistry) {
    let running = [
        modules.query_running(registry, "AutoShop"),
        modules.query_running(registry, "AutoBuild"),
        modules.query_running(registry, "AutoFarm"),
        modules.query_running(registry, "CrystalAura"),
        modules.query_running(registry, "Fucker"),
        modules.query_running(registry, "AutoTrap"),
        modules.query_running(registry, "BlockTrap"),
        modules.query_running(registry, "Nuker"),
        modules.query_running(registry, "TickBase"),
        modules.query_running(registry, "Backtrack"),
        modules.query_running(registry, "Fly"),
        modules.query_running(registry, "LongJump"),
    ];
    modules.auto_shop.tick(running[0]);
    modules.auto_build.tick(running[1]);
    modules.auto_farm.tick(running[2]);
    modules.crystal_aura.tick(running[3]);
    modules.fucker.tick(running[4]);
    modules.auto_trap.tick(running[5]);
    modules.block_trap.tick(running[6]);
    modules.nuker.tick(running[7]);
    modules.tick_base.tick(running[8]);
    modules.backtrack.tick(running[9]);
    modules.fly.tick(running[10]);
    modules.long_jump.tick(running[11]);
}
