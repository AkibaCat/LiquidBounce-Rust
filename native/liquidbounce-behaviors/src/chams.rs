// SPDX-License-Identifier: GPL-3.0-or-later
// ModuleChams.kt and chams.frag, LiquidBounce ae0eb396.
// Copyright (c) 2015-2026 CCBlueX.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChamsMode {
    #[default]
    Normal,
    Image,
}
impl ChamsMode {
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Image => "Image",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageMapping {
    #[default]
    Repeat,
    Stretch,
    Cover,
}
impl ImageMapping {
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Repeat => "Repeat",
            Self::Stretch => "Stretch",
            Self::Cover => "Cover",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageFiltering {
    #[default]
    Nearest,
    Linear,
}
impl ImageFiltering {
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Nearest => "Nearest",
            Self::Linear => "Linear",
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Chams {
    pub mode: ChamsMode,
    /// FileValue's normalized client-folder-relative path. Empty means the root folder.
    pub file: String,
    pub mapping: ImageMapping,
    pub tile_width: i32,
    pub filtering: ImageFiltering,
    pub offset: [f32; 2],
}
impl Default for Chams {
    fn default() -> Self {
        Self {
            mode: ChamsMode::Normal,
            file: String::new(),
            mapping: ImageMapping::Repeat,
            tile_width: 256,
            filtering: ImageFiltering::Nearest,
            offset: [0.0; 2],
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ImageUniform {
    pub scale: [f32; 2],
    pub offset: [f32; 2],
}
impl Chams {
    /// Preserve the Kotlin float operation order, including out-of-UI-range imports.
    pub fn image_uniform(&self, target: [i32; 2], image: [i32; 2]) -> ImageUniform {
        let mut uniform = match self.mapping {
            ImageMapping::Repeat => {
                let width = self.tile_width as f32;
                ImageUniform {
                    scale: [width, width * image[1] as f32 / image[0] as f32],
                    offset: [0.0; 2],
                }
            }
            ImageMapping::Stretch => ImageUniform {
                scale: target.map(|v| v as f32),
                offset: [0.0; 2],
            },
            ImageMapping::Cover => {
                let scale =
                    (target[0] as f32 / image[0] as f32).max(target[1] as f32 / image[1] as f32);
                let width = image[0] as f32 * scale;
                let height = image[1] as f32 * scale;
                ImageUniform {
                    scale: [width, height],
                    offset: [
                        (target[0] as f32 - width) * 0.5,
                        (target[1] as f32 - height) * 0.5,
                    ],
                }
            }
        };
        for (offset, configured) in uniform.offset.iter_mut().zip(self.offset) {
            *offset += configured;
        }
        uniform
    }
    /// Actor-to-renderer payload; resolve FileValue relative to ConfigSystem.rootFolder.
    pub fn render_settings(&self, root: &std::path::Path) -> String {
        let path = std::path::Path::new(&self.file);
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };
        serde_json::json!({"mode":self.mode.tag(),"file":path,"mapping":self.mapping.tag(),
            "tileWidth":self.tile_width,"filtering":self.filtering.tag(),"offset":self.offset})
        .to_string()
    }
}
pub const SUPPORTED_RENDER_TYPES: [&str; 13] = [
    "armor_cutout_no_cull",
    "armor_decal_cutout_no_cull",
    "armor_entity_glint",
    "entity_translucent",
    "entity_cutout",
    "entity_cutout_cull",
    "entity_cutout_no_cull",
    "entity_solid",
    "entity_glint",
    "glint",
    "glint_translucent",
    "item_cutout",
    "item_translucent",
];

/// chams.frag compares reverse-Z values: visible texels retain their entity texture.
/// A transparent entity texel is discarded, even if the image itself is opaque.
pub fn image_fragment(
    entity: [f32; 4],
    entity_depth: f32,
    scene_depth: f32,
    image: [f32; 4],
) -> Option<[f32; 4]> {
    if entity[3] == 0.0 {
        None
    } else if entity_depth > scene_depth + 0.00001 {
        Some(entity)
    } else {
        Some(image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mapping_preserves_aspect_centering_and_additive_offsets() {
        let mut state = Chams {
            offset: [7.0, -3.0],
            ..Default::default()
        };
        assert_eq!(
            state.image_uniform([1920, 1080], [400, 200]),
            ImageUniform {
                scale: [256.0, 128.0],
                offset: [7.0, -3.0]
            }
        );
        state.mapping = ImageMapping::Stretch;
        assert_eq!(
            state.image_uniform([1920, 1080], [400, 200]),
            ImageUniform {
                scale: [1920.0, 1080.0],
                offset: [7.0, -3.0]
            }
        );
        state.mapping = ImageMapping::Cover;
        assert_eq!(
            state.image_uniform([1920, 1080], [400, 200]),
            ImageUniform {
                scale: [2160.0, 1080.0],
                offset: [-113.0, -3.0]
            }
        );
        assert_eq!(
            state.image_uniform([1920, 1080], [200, 400]),
            ImageUniform {
                scale: [1920.0001, 3840.0002],
                offset: [6.999939, -1383.0001]
            }
        );
    }
    #[test]
    fn shader_uses_the_exact_depth_epsilon_and_alpha_gate() {
        let entity = [0.2, 0.3, 0.4, 0.5];
        let image = [0.9, 0.8, 0.7, 1.0];
        assert_eq!(image_fragment(entity, 0.8, 0.2, image), Some(entity));
        assert_eq!(image_fragment(entity, 0.2, 0.8, image), Some(image));
        assert_eq!(image_fragment(entity, 0.50001, 0.5, image), Some(image));
        assert_eq!(image_fragment([0.0; 4], 0.8, 0.2, image), None);
    }
}
