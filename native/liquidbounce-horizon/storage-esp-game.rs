// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_client::{block_entity_render::Scene as BlockEntityScene, mesher::Vertex};
use serde_json::{Value, json};
fn storage_block(n: &str) -> bool {
    matches!(
        n,
        "minecraft:chest"
            | "minecraft:trapped_chest"
            | "minecraft:barrel"
            | "minecraft:ender_chest"
            | "minecraft:furnace"
            | "minecraft:blast_furnace"
            | "minecraft:smoker"
            | "minecraft:brewing_stand"
            | "minecraft:dispenser"
            | "minecraft:dropper"
            | "minecraft:crafter"
            | "minecraft:hopper"
            | "minecraft:decorated_pot"
            | "minecraft:chiseled_bookshelf"
            | "minecraft:shulker_box"
    ) || n.ends_with("_shulker_box")
        || n.ends_with("_shelf")
}
fn entity(
    id: u32,
    name: &str,
    chested: bool,
    p: horizon_core::geometry::Vec3,
    old: horizon_core::geometry::Vec3,
    width: f64,
    height: f64,
    alpha: f64,
) -> Value {
    let at = [p.x, p.y, p.z];
    let previous = [old.x, old.y, old.z];
    json!({"id":id as i32,"name":name,"chested":chested,"position":at,"interpolated":std::array::from_fn::<_,3,_>(|i|previous[i]+(at[i]-previous[i])*alpha),"width":width,"height":height})
}
impl Game<'_> {
    pub(super) fn client_storage_esp_frame(&self, camera: glam::DVec3, alpha: f64) {
        horizon_client::gfx::storage_esp::begin_frame("null");
        horizon_client::gfx::storage_plain::begin_frame("null");
        if !self.native_window() || self.player.is_none() {
            return;
        }
        let blocks=(&self.world.block_entities).into_iter().filter_map(|(&(x,y,z),_)|{let position=[x,y,z];let state=self.world.block(x,y,z);let name=&self.res.facts().get(state).name;if !storage_block(name){return None;}Some(json!({"position":position,"name":name.as_ref(),"model":self.res.block_entity_render().renders(state.0),"boxes":self.outline_boxes(position)}))}).collect::<Vec<_>>();
        let mut entities = vec![];
        for m in self.mobs.iter().filter(|m| {
            !m.removed
                && m.extra.chested
                && matches!(
                    &*m.kind.id,
                    "minecraft:donkey"
                        | "minecraft:mule"
                        | "minecraft:llama"
                        | "minecraft:trader_llama"
                )
        }) {
            let (w, h, _) = m.dims();
            entities.push(entity(
                m.id, &m.kind.id, true, m.position, m.previous, w, h, alpha,
            ));
        }
        for b in self.boats.iter().filter(|b| b.chest) {
            let bounds = b.bounds();
            let lo = bounds.min_position();
            let hi = bounds.max_position();
            entities.push(entity(
                b.id,
                if &*b.wood == "bamboo" {
                    "minecraft:chest_raft"
                } else {
                    "minecraft:chest_boat"
                },
                true,
                b.position,
                b.previous,
                hi.x - lo.x,
                hi.y - lo.y,
                alpha,
            ));
        }
        for c in self.carts.iter().filter(|c| {
            matches!(
                c.kind,
                super::minecarts::CartKind::Chest | super::minecarts::CartKind::Hopper
            )
        }) {
            let bounds = c.bounds();
            let lo = bounds.min_position();
            let hi = bounds.max_position();
            entities.push(entity(
                c.id,
                c.kind.id(),
                true,
                c.position,
                c.previous,
                hi.x - lo.x,
                hi.y - lo.y,
                alpha,
            ));
        }
        let eye = (self.camera_orientation() * -glam::Vec3::Z).to_array();
        let input =
            json!({"camera":camera.to_array(),"eye":eye,"blocks":blocks,"entities":entities})
                .to_string();
        let mut output = String::new();
        self.native_client_storage_esp(&input, &mut output);
        let Ok(v) = serde_json::from_str::<Value>(&output) else {
            return;
        };
        horizon_client::gfx::storage_esp::begin_frame(&v["body"].to_string());
        horizon_client::gfx::storage_plain::begin_frame(&v["plain"].to_string());
        let lid = |p: [i32; 3]| self.chest_lid(p, alpha as f32);
        let anim = |p: [i32; 3]| self.block_anim(p, alpha as f32);
        let scene = BlockEntityScene {
            camera,
            orientation: self.camera_orientation(),
            anim: &anim,
            texts: None,
            partial: alpha as f32,
            game_time: self.world.game_time,
            max_y: self.world.shape.max_y() - 1,
            world: &self.world,
            sprites: &self.gui.atlas.sprites,
            emissive: &self.gui.atlas.emissive,
            translucent_sprites: &self.gui.atlas.translucent,
            atlas_revision: self.gui.atlas.revision,
            nether: self.nether_lighting(),
            lid: &lid,
            sign: &|_| None,
            banner: &|_| Vec::new(),
            skull: &|_| None,
            glyph: &|c| self.gui.atlas.glyph_cell(c),
            glyphs: self.gui.atlas.font.epoch(),
            font: Some(&self.gui.atlas.font),
            dynamic_visible: None,
        };
        for model in v["models"].as_array().into_iter().flatten() {
            let Ok(p) = serde_json::from_value::<[i32; 3]>(model["position"].clone()) else {
                continue;
            };
            let Some(color) = model["color"].as_u64() else {
                continue;
            };
            let block = (p, self.world.block(p[0], p[1], p[2]).0);
            let mut vertices = Vec::<Vertex>::new();
            self.res
                .block_entity_render()
                .render(&scene, std::iter::once(&block), &mut vertices);
            horizon_client::gfx::storage_esp::capture_model(&vertices, color as u32);
        }
    }
}
