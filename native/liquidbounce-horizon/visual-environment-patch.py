#!/usr/bin/env python3
"""AntiBlind's original 29 visual switches at Horizon's real rendering/effect sites.

The original SkyLightEngine mixin has no side guard. Its increase pass is gated
on both client and integrated-server light workers; decreases and block light
are preserved. This intentionally retains that original single-player effect.
"""

TAGS = ('Blinding Darkness Nausea Armor MobInSpawner EnchantTableBook EatParticles '
        'BlockBreakParticles BlockBreakOverlay Title PumpkinBlur LiquidsFog PowderSnowFog '
        'FloatingItems MapContents MapMarkers PortalOverlay WallOverlay FallingBlocks '
        'BeaconBeams SkylightUpdates GuiBackground SpyglassOverlay SignText '
        'InvisibleEntities BossBars ExplosionParticles WorldBorder FallingLeaves').split()


def policy_source():
    variants = ', '.join(TAGS)
    return '''// SPDX-License-Identifier: GPL-3.0-or-later
//! Atomic visual snapshot published by the native client before ticking/rendering.
//! One word keeps selections, raw Kotlin Int opacity and both flags consistent
//! across the render/lighting workers. Existing particles are never cleared.
use std::sync::atomic::{AtomicU64, Ordering};
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Choice { ''' + variants + ''' }
pub const ALL: u32 = (1 << 29) - 1;
const DEFAULT: u64 = ALL as u64 | (100u64 << 29);
static POLICY: AtomicU64 = AtomicU64::new(DEFAULT);
pub fn publish(mask: u32, fire_opacity: i32, hide_particles: bool, skip_terrain: bool) {
    let bits = u64::from(mask & ALL) | (u64::from(fire_opacity as u32) << 29)
        | (u64::from(hide_particles) << 61) | (u64::from(skip_terrain) << 62);
    POLICY.store(bits, Ordering::Release);
}
pub fn allows(choice: Choice) -> bool {
    POLICY.load(Ordering::Acquire) & (1u64 << choice as u8) != 0
}
pub fn hide_particles() -> bool { POLICY.load(Ordering::Acquire) & (1 << 61) != 0 }
pub fn skip_terrain() -> bool { POLICY.load(Ordering::Acquire) & (1 << 62) != 0 }
pub fn fire_alpha() -> f32 {
    let opacity = (POLICY.load(Ordering::Acquire) >> 29) as u32 as i32;
    let factor = opacity as f32 / 100.0;
    // ARGB.multiplyAlpha truncates the byte product, then ARGB.color shifts
    // that Int into the alpha byte. Keep raw imported values' Java cast behavior.
    let alpha = (0xE5 as f32 * factor) as i32 as u32 & 255;
    alpha as f32 / 255.0
}
'''


def apply(edit, replace_once):
    def at_head(text, anchor, statement):
        if text.count(anchor) != 1:
            raise ValueError(f'Expected one visual function: {anchor}')
        opening = text.index('{', text.index(anchor)) + 1
        return text[:opening] + '\n        ' + statement + text[opening:]

    def gate(path, anchor, choice, result='return;'):
        scope = 'crate' if '/horizon-client-render/' in path else 'horizon_client'
        edit(path, lambda text: at_head(text, anchor,
            f'if !{scope}::visual_environment::allows({scope}::visual_environment::Choice::{choice}) {{ {result} }}'))

    def native(text):
        anchor = '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
        return replace_once(text, anchor, '''    /// Query original AntiBlind options once before effects and worker dispatch.
    pub(super) fn sync_native_visual_environment(&self) {
        let (mut render_mask, mut fire_opacity, mut hide_particles, mut skip_terrain_loading) =
            (horizon_client::visual_environment::ALL, 100, false, false);
        self.native_client_visual_environment_policy(&mut render_mask, &mut fire_opacity,
            &mut hide_particles, &mut skip_terrain_loading);
        horizon_client::visual_environment::publish(render_mask, fire_opacity,
            hide_particles, skip_terrain_loading);
    }
    pub(super) fn native_client_visual_environment_policy(&self, render_mask: &mut u32,
        fire_opacity: &mut i32, hide_particles: &mut bool, skip_terrain_loading: &mut bool) {
        if self.native_window() {
            mod_api::dispatch_client_visual_environment_policy(self, mod_api::mods(),
                render_mask, fire_opacity, hide_particles, skip_terrain_loading);
        }
    }

''' + anchor)

    def game(text):
        text = at_head(text, '    fn render_frame(&mut self) -> Result<bool>', 'self.sync_native_visual_environment();')
        text = at_head(text, '    fn tick(&mut self) -> Result<()>', 'self.sync_native_visual_environment();')
        return replace_once(text, '''                self.potions.effects.get("blindness").is_some()
                    || self.potions.effects.get("darkness").is_some(),''', '''                (horizon_client::visual_environment::allows(horizon_client::visual_environment::Choice::Blinding)
                    && self.potions.effects.get("blindness").is_some())
                    || (horizon_client::visual_environment::allows(horizon_client::visual_environment::Choice::Darkness)
                        && self.potions.effects.get("darkness").is_some()),''')

    edit('crates/horizon-client-render/src/visual_environment.rs', lambda _: policy_source())
    edit('crates/horizon-client-render/src/lib.rs', lambda t: replace_once(t, 'pub mod fog;\n', 'pub mod fog;\npub mod visual_environment;\n'))
    edit('crates/horizon-client/src/client_lib.rs', lambda t: replace_once(t, '    weather, widgets, world, worldborder, zip,\n', '    visual_environment, weather, widgets, world, worldborder, zip,\n'))
    edit('crates/horizon-client/src/game/native_hooks.rs', native)
    edit('crates/horizon-client/src/game.rs', game)

    def fog(text):
        # Filter effect environments without deleting effects from gameplay.
        text = at_head(text, 'pub fn setup_fog(', '''let mut filtered;
    let input = if !crate::visual_environment::allows(crate::visual_environment::Choice::Blinding)
        || !crate::visual_environment::allows(crate::visual_environment::Choice::Darkness) {
        filtered = FogInputs { ..*input };
        if !crate::visual_environment::allows(crate::visual_environment::Choice::Blinding) { filtered.blindness = None; }
        if !crate::visual_environment::allows(crate::visual_environment::Choice::Darkness) { filtered.darkness = None; }
        &filtered
    } else { input };''')
        return replace_once(text, '''    fog
}
/// Horizon's Fog option''', '''    // Only the selected fluid environment is changed. Blindness/darkness
    // take priority over water, whereas lava and powder snow take priority.
    let clear_environment = match input.fog_type {
        FogType::Lava => !crate::visual_environment::allows(crate::visual_environment::Choice::LiquidsFog),
        FogType::Water if input.blindness.is_none() && input.darkness.is_none() =>
            !crate::visual_environment::allows(crate::visual_environment::Choice::LiquidsFog),
        FogType::PowderSnow => !crate::visual_environment::allows(crate::visual_environment::Choice::PowderSnowFog),
        _ => false,
    };
    if clear_environment {
        fog.environmental_start = f32::MAX;
        fog.environmental_end = f32::MAX;
        fog.sky_end = f32::MAX;
        fog.cloud_end = f32::MAX;
    }
    fog
}
/// Horizon's Fog option''')

    edit('crates/horizon-client-render/src/fog.rs', fog)
    edit('crates/horizon-client/src/game/sculk.rs', lambda t: replace_once(
        at_head(t, '    pub(super) fn darkness_fog(',
            'if !horizon_client::visual_environment::allows(horizon_client::visual_environment::Choice::Darkness) { return None; }'),
        '        let scale = self.settings.file.double("darknessEffectScale") as f32;\n',
        '        let scale = if horizon_client::visual_environment::allows(horizon_client::visual_environment::Choice::Darkness) { self.settings.file.double("darknessEffectScale") as f32 } else { 0.0 };\n'))

    def fov(text):
        # Math.max(portal, nausea) is cancelled as a whole upstream.
        return at_head(text, '    pub(super) fn spinning_matrix(',
            'if !horizon_client::visual_environment::allows(horizon_client::visual_environment::Choice::Nausea) { return glam::Mat4::IDENTITY; }')
    edit('crates/horizon-client/src/game/fov.rs', fov)

    def screens(text):
        text = replace_once(text, '''        if let Some(name) = &input.block_overlay
''', '''        if crate::visual_environment::allows(crate::visual_environment::Choice::WallOverlay)
            && let Some(name) = &input.block_overlay
''')
        return replace_once(text, '                let alpha = 0xE5 as f32 / 255.0;\n',
            '                let alpha = crate::visual_environment::fire_alpha();\n')
    edit('crates/horizon-client-render/src/screen_effects.rs', screens)

    def hud(text):
        for anchor, choice in [('    pub fn title(', 'Title'), ('pub fn boss_overlay(', 'BossBars'),
                                ('pub fn spyglass(', 'SpyglassOverlay'), ('pub fn confusion_overlay(', 'Nausea')]:
            text = at_head(text, anchor, f'if !crate::visual_environment::allows(crate::visual_environment::Choice::{choice}) {{ return Vec::new(); }}')
        text = replace_once(text, '''            for texture in &i.equipment {
''', '''            for texture in &i.equipment {
                if matches!(texture.as_str(), "misc/pumpkinblur" | "minecraft:misc/pumpkinblur")
                    && !crate::visual_environment::allows(crate::visual_environment::Choice::PumpkinBlur) { continue; }
''')
        text = replace_once(text, '    if i.ticks_frozen > 0 {\n',
            '    if i.ticks_frozen > 0 && crate::visual_environment::allows(crate::visual_environment::Choice::PowderSnowFog) {\n')
        return replace_once(text, '        out.push(portal_overlay(w, h, portal));\n',
            '        if crate::visual_environment::allows(crate::visual_environment::Choice::PortalOverlay) { out.push(portal_overlay(w, h, portal)); }\n')
    edit('crates/horizon-client-render/src/hud_overlays.rs', hud)

    def particles(text):
        return at_head(text, '    pub fn add_with(', '''if matches!(options.ty, ParticleType::Explosion | ParticleType::ExplosionEmitter)
            && !crate::visual_environment::allows(crate::visual_environment::Choice::ExplosionParticles) { return; }''')
    edit('crates/horizon-client-render/src/particles.rs', particles)
    # The provider runs before ParticleEngine.add(Particle)'s admission check,
    # retaining original random draws; already admitted particles keep ticking.
    edit('crates/horizon-client-render/src/particles/spawn.rs', lambda t: replace_once(t,
        '            self.pending.push(particle);\n',
        '            if !crate::visual_environment::hide_particles() { self.pending.push(particle); }\n'))

    for method in ('destroy_particles(', 'crack_particle('):
        gate('crates/horizon-client/src/game/block_particles.rs', '    pub(super) fn ' + method, 'BlockBreakParticles')
    edit('crates/horizon-client/src/animate_tick.rs', lambda t: replace_once(t,
        '            if let Some((ty, chance)) = falling\n',
        '            if crate::visual_environment::allows(crate::visual_environment::Choice::FallingLeaves)\n                && let Some((ty, chance)) = falling\n'))

    def item_particles(text):
        return at_head(text, '    pub(super) fn eat_particles(', '''if !horizon_client::visual_environment::allows(horizon_client::visual_environment::Choice::EatParticles)
            && self.pack.item_renderer.models.components(&stack_view(stack)).get("minecraft:food").is_some() { return; }''')
    edit('crates/horizon-client/src/game/item_particles.rs', item_particles)
    edit('crates/horizon-client/src/game/remote.rs', lambda t: at_head(t, '    fn remote_player_eating(',
        '''if !horizon_client::visual_environment::allows(horizon_client::visual_environment::Choice::EatParticles)
            && self.res.items().get(item).is_some_and(|info| info.food.is_some()) { return; }'''))
    # A custom consumable with no FOOD component retains its particles/sound.
    def fox(text):
        text = replace_once(text, '    let _ = consumable.emit(&mut Eater { m, events }, (), stack, 16);\n',
            '''    let food = w.items.and_then(|i| i.items.get(item)).is_some_and(|i| i.food.is_some());
    let _ = consumable.emit(&mut Eater { m, events, food }, (), stack, 16);
''')
        text = replace_once(text, "struct Eater<'a> {\n", "struct Eater<'a> {\n    food: bool,\n")
        return at_head(text, '    fn spawn_item_particles(',
            'if self.food && !crate::visual_environment::allows(crate::visual_environment::Choice::EatParticles) { return Ok(()); }')
    edit('crates/horizon-client/src/mobs/fox.rs', fox)

    def totem(text):
        return replace_once(text, '''            self.item_activation
                .activate(totem, self.particles.random());''', '''            if horizon_client::visual_environment::allows(horizon_client::visual_environment::Choice::FloatingItems) {
                self.item_activation.activate(totem, self.particles.random());
            }''')
    edit('crates/horizon-client/src/game/totem.rs', totem)

    def maps(text):
        text = at_head(text, '    pub(super) fn map_vertices(',
            'if !horizon_client::visual_environment::allows(horizon_client::visual_environment::Choice::MapContents) { return; }')
        return replace_once(text, '        let decorations = self.maps.data.get(&id).map_or(&[][..], |m| &m.decorations);\n',
            '''        if !horizon_client::visual_environment::allows(horizon_client::visual_environment::Choice::MapMarkers) { return; }
        let decorations = self.maps.data.get(&id).map_or(&[][..], |m| &m.decorations);
''')
    edit('crates/horizon-client/src/game/maps.rs', maps)
    gate('crates/horizon-client/src/game/falling_blocks.rs', '    pub(super) fn falling_model_vertices(', 'FallingBlocks')
    gate('crates/horizon-client/src/game/beacon.rs', '    pub(super) fn beacon_geometry(', 'BeaconBeams')
    gate('crates/horizon-client-render/src/breaking.rs', 'pub fn geometry(', 'BlockBreakOverlay')
    gate('crates/horizon-client/src/game/screen_backgrounds.rs', '    pub(super) fn render_transparent_background(', 'GuiBackground')
    gate('crates/horizon-client/src/game/screen_backgrounds.rs', '    pub(super) fn border_draw(', 'WorldBorder', 'return None;')

    def spawners(text):
        return replace_once(text, '''                Some(BlockEntity::Spawner(s)) => {
                    s.display_pose''', '''                Some(BlockEntity::Spawner(s)) => {
                    if !horizon_client::visual_environment::allows(horizon_client::visual_environment::Choice::MobInSpawner) { continue; }
                    s.display_pose''')
    edit('crates/horizon-client/src/game/spawner.rs', spawners)

    def block_entities(text):
        text = at_head(text, 'fn sign_text_calls(',
            'if !crate::visual_environment::allows(crate::visual_environment::Choice::SignText) { return Vec::new(); }')
        # A cached glyph run must also disappear immediately when toggled.
        text = at_head(text, '    fn sign_text(',
            'if !crate::visual_environment::allows(crate::visual_environment::Choice::SignText) { return; }')
        text = at_head(text, '    pub fn render_breaking(',
            'if !crate::visual_environment::allows(crate::visual_environment::Choice::BlockBreakOverlay) { return; }')
        return replace_once(text, '''            Kind::EnchantingTable => {
                let book''', '''            Kind::EnchantingTable => {
                if !crate::visual_environment::allows(crate::visual_environment::Choice::EnchantTableBook) { return; }
                let book''')
    edit('crates/horizon-client-render/src/block_entity_render.rs', block_entities)

    def armor(text):
        text = at_head(text, '    pub(super) fn player_armor_layers(',
            'if !horizon_client::visual_environment::allows(horizon_client::visual_environment::Choice::Armor) { return Vec::new(); }')
        text = at_head(text, '    pub(super) fn worn_armor_layers(',
            'if &*m.kind.id == "minecraft:player" && !horizon_client::visual_environment::allows(horizon_client::visual_environment::Choice::Armor) { return Vec::new(); }')
        return at_head(text, '    fn armor_stand_vertices(',
            'if a.invisible && !horizon_client::visual_environment::allows(horizon_client::visual_environment::Choice::InvisibleEntities) { return; }')
    edit('crates/horizon-client/src/game/armor_stands.rs', armor)
    edit('crates/horizon-client/src/game/mob_blocks.rs', lambda t: at_head(t, '    pub(super) fn head_item_geometry(',
        'if &*mob.kind.id == "minecraft:player" && !horizon_client::visual_environment::allows(horizon_client::visual_environment::Choice::Armor) { return; }'))
    edit('crates/horizon-client/src/game/creatures/mob_geometry.rs', lambda t: replace_once(t,
        '        for mob in &self.mobs {\n', '''        for mob in &self.mobs {
            if horizon_client::mobs::trader::is_invisible(mob)
                && !horizon_client::visual_environment::allows(horizon_client::visual_environment::Choice::InvisibleEntities) { continue; }
'''))
    def players(text):
        for anchor in ('    pub(super) fn player_model_geometry(', '    pub(super) fn draw_player_preview('):
            text = at_head(text, anchor,
                'if self.own_invisible() && !horizon_client::visual_environment::allows(horizon_client::visual_environment::Choice::InvisibleEntities) { return; }')
        return text
    edit('crates/horizon-client/src/game/perspective.rs', players)
    edit('crates/horizon-client/src/mobs/render.rs', lambda t: at_head(t, 'fn render_visible(',
        'if super::trader::is_invisible(m) && !crate::visual_environment::allows(crate::visual_environment::Choice::InvisibleEntities) { return; }'))

    def lighting(text):
        text = replace_once(text, '    spread(sky, sky_queue);\n',
            '    if crate::visual_environment::allows(crate::visual_environment::Choice::SkylightUpdates) { spread(sky, sky_queue); }\n')
        return replace_once(text, '    while let Some(p) = increase.pop_front() {\n',
            '''    while let Some(p) = increase.pop_front() {
        if sky && !crate::visual_environment::allows(crate::visual_environment::Choice::SkylightUpdates) { continue; }
''')
    edit('crates/horizon-client-render/src/light.rs', lighting)

    def loading(text):
        text = replace_once(text, '            if ready {\n',
            '            if ready || crate::visual_environment::skip_terrain() {\n')
        return replace_once(text,
            '        matches!(self.state, Some(ClientState::Ready { ready_at }) if now >= ready_at + self.close_delay_ms)\n',
            '        matches!(self.state, Some(ClientState::Ready { ready_at }) if now >= ready_at + if crate::visual_environment::skip_terrain() { 0 } else { self.close_delay_ms })\n')
    edit('crates/horizon-client-render/src/level_loading.rs', loading)
    return {'hook_names': ['client_visual_environment_policy'], 'helper_names': []}
