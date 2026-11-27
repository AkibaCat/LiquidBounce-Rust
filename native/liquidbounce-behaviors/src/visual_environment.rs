// SPDX-License-Identifier: GPL-3.0-or-later
// ModuleAntiBlind.kt at LiquidBounce ae0eb396, CCBlueX 2015-2026.

/// Stable enum order and tags match the upstream MultiEnumChoice serializer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DoRender {
    Blinding,
    Darkness,
    Nausea,
    Armor,
    MobInSpawner,
    EnchantTableBook,
    EatParticles,
    BlockBreakParticles,
    BlockBreakOverlay,
    Title,
    PumpkinBlur,
    LiquidsFog,
    PowderSnowFog,
    FloatingItems,
    MapContents,
    MapMarkers,
    PortalOverlay,
    WallOverlay,
    FallingBlocks,
    BeaconBeams,
    SkylightUpdates,
    GuiBackground,
    SpyglassOverlay,
    SignText,
    InvisibleEntities,
    BossBars,
    ExplosionParticles,
    WorldBorder,
    FallingLeaves,
}

impl DoRender {
    pub const ALL: [Self; 29] = [
        Self::Blinding,
        Self::Darkness,
        Self::Nausea,
        Self::Armor,
        Self::MobInSpawner,
        Self::EnchantTableBook,
        Self::EatParticles,
        Self::BlockBreakParticles,
        Self::BlockBreakOverlay,
        Self::Title,
        Self::PumpkinBlur,
        Self::LiquidsFog,
        Self::PowderSnowFog,
        Self::FloatingItems,
        Self::MapContents,
        Self::MapMarkers,
        Self::PortalOverlay,
        Self::WallOverlay,
        Self::FallingBlocks,
        Self::BeaconBeams,
        Self::SkylightUpdates,
        Self::GuiBackground,
        Self::SpyglassOverlay,
        Self::SignText,
        Self::InvisibleEntities,
        Self::BossBars,
        Self::ExplosionParticles,
        Self::WorldBorder,
        Self::FallingLeaves,
    ];
    pub const TAGS: [&'static str; 29] = [
        "Blinding",
        "Darkness",
        "Nausea",
        "Armor",
        "MobInSpawner",
        "EnchantTableBook",
        "EatParticles",
        "BlockBreakParticles",
        "BlockBreakOverlay",
        "Title",
        "PumpkinBlur",
        "LiquidsFog",
        "PowderSnowFog",
        "FloatingItems",
        "MapContents",
        "MapMarkers",
        "PortalOverlay",
        "WallOverlay",
        "FallingBlocks",
        "BeaconBeams",
        "SkylightUpdates",
        "GuiBackground",
        "SpyglassOverlay",
        "SignText",
        "InvisibleEntities",
        "BossBars",
        "ExplosionParticles",
        "WorldBorder",
        "FallingLeaves",
    ];
    pub const ALL_MASK: u32 = (1 << 29) - 1;
    pub const DEFAULT_MASK: u32 = Self::ALL_MASK
        & !(Self::Blinding.bit()
            | Self::Darkness.bit()
            | Self::Nausea.bit()
            | Self::PumpkinBlur.bit()
            | Self::LiquidsFog.bit()
            | Self::PowderSnowFog.bit()
            | Self::FloatingItems.bit()
            | Self::PortalOverlay.bit()
            | Self::WallOverlay.bit());
    pub const fn bit(self) -> u32 {
        1 << self as u8
    }
    pub const fn tag(self) -> &'static str {
        Self::TAGS[self as usize]
    }
    pub fn from_tag(tag: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|choice| choice.tag() == tag)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AntiBlind {
    pub render_mask: u32,
    pub fire_opacity: i32,
    pub no_particles: bool,
    pub skip_terrain_loading: bool,
}
impl Default for AntiBlind {
    fn default() -> Self {
        Self {
            render_mask: DoRender::DEFAULT_MASK,
            fire_opacity: 100,
            no_particles: false,
            skip_terrain_loading: false,
        }
    }
}
impl AntiBlind {
    pub fn can_render(self, running: bool, choice: DoRender) -> bool {
        !running || self.render_mask & choice.bit() != 0
    }
    pub fn fire_opacity_percentage(self, running: bool) -> f32 {
        if running {
            self.fire_opacity as f32 / 100.0
        } else {
            1.0
        }
    }
    /// Effective client policy. Disabled modules leave every vanilla visual intact.
    pub fn policy(self, running: bool) -> (u32, i32, bool, bool) {
        if running {
            (
                self.render_mask & DoRender::ALL_MASK,
                self.fire_opacity,
                self.no_particles,
                self.skip_terrain_loading,
            )
        } else {
            (DoRender::ALL_MASK, 100, false, false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn upstream_default_selection_and_disabled_state() {
        let defaults = AntiBlind::default();
        assert_eq!(defaults.render_mask.count_ones(), 20);
        for choice in DoRender::ALL {
            assert_eq!(DoRender::from_tag(choice.tag()), Some(choice));
            assert!(defaults.can_render(false, choice));
        }
        assert!(!defaults.can_render(true, DoRender::Nausea));
        assert!(defaults.can_render(true, DoRender::FallingLeaves));
        assert_eq!(
            defaults.policy(false),
            (DoRender::ALL_MASK, 100, false, false)
        );
    }
    #[test]
    fn imported_values_keep_upstream_arithmetic() {
        let options = AntiBlind {
            fire_opacity: -25,
            no_particles: true,
            skip_terrain_loading: true,
            render_mask: 0,
        };
        assert_eq!(options.fire_opacity_percentage(true), -0.25);
        assert_eq!(options.fire_opacity_percentage(false), 1.0);
        assert_eq!(options.policy(true), (0, -25, true, true));
        assert_eq!(DoRender::from_tag("Unknown"), None);
    }
}
