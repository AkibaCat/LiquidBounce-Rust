// SPDX-License-Identifier: GPL-3.0-or-later
// FontManager.themeFontFace: actual loaded SFNT names, active theme then included theme.
use sha2::Digest as _;
#[derive(Clone)]
struct TargetFontFace {
    name: String,
    font: Arc<ab_glyph::FontVec>,
    hash: [u8; 32],
    bold: bool,
}
struct TargetFontRegistry {
    faces: HashMap<String, TargetFontFace>,
    primary: String,
    generation: u64,
}
fn target_face(bytes: Vec<u8>) -> Result<TargetFontFace, String> {
    let parsed = ttf_parser::Face::parse(&bytes, 0)
        .map_err(|e| format!("Invalid TrueType/OpenType face: {e:?}"))?;
    let bold = parsed.is_bold();
    let name = parsed
        .names()
        .into_iter()
        .filter(|n| n.name_id == ttf_parser::name_id::FULL_NAME)
        .filter_map(|n| {
            let text = n.to_string().or_else(|| {
                n.name
                    .is_ascii()
                    .then(|| String::from_utf8_lossy(n.name).into_owned())
            })?;
            Some((
                (
                    u8::from(n.language_id == 0x409),
                    u8::from(n.platform_id == ttf_parser::PlatformId::Windows),
                ),
                text,
            ))
        })
        .max_by_key(|(score, _)| *score)
        .map(|(_, name)| name)
        .ok_or_else(|| "Font has no readable full face name".to_owned())?;
    let hash = sha2::Sha256::digest(&bytes).into();
    let font = ab_glyph::FontVec::try_from_vec(bytes)
        .map_err(|e| format!("Unsupported font face: {e:?}"))?;
    Ok(TargetFontFace {
        name,
        font: Arc::new(font),
        hash,
        bold,
    })
}
fn target_registry() -> &'static std::sync::RwLock<TargetFontRegistry> {
    static FONTS: std::sync::OnceLock<std::sync::RwLock<TargetFontRegistry>> =
        std::sync::OnceLock::new();
    FONTS.get_or_init(|| {
        let faces = TARGET_FONTS
            .iter()
            .map(|encoded| {
                target_face(crate::skins::base64_decode(encoded).expect("included font bytes"))
                    .expect("pinned included TrueType font")
            })
            .map(|f| (f.name.clone(), f))
            .collect();
        std::sync::RwLock::new(TargetFontRegistry {
            faces,
            primary: "Inter Medium".into(),
            generation: 1,
        })
    })
}
fn target_font_generation() -> u64 {
    target_registry()
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .generation
}
fn target_font_signature(registry: &TargetFontRegistry) -> (Option<[u8; 32]>, Option<[u8; 32]>) {
    (
        registry.faces.get(&registry.primary).map(|f| f.hash),
        (registry.primary == "Inter Medium")
            .then(|| registry.faces.get("Inter Bold").map(|f| f.hash))
            .flatten(),
    )
}
/// The browser passes the real active theme root, including custom local themes.
/// The names come from each font's name table, never from its file name.
pub fn liquidbounce_load_theme_fonts(root: &Path) -> Result<(), String> {
    let metadata: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("metadata.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let mut loaded = Vec::new();
    for name in metadata["fonts"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
    {
        let relative = Path::new(name);
        if relative
            .components()
            .any(|p| !matches!(p, std::path::Component::Normal(_)))
        {
            eprintln!("[LiquidBounce] Invalid theme font resource: {name}");
            continue;
        }
        match std::fs::read(root.join("fonts").join(relative))
            .map_err(|e| e.to_string())
            .and_then(target_face)
        {
            Ok(face) => loaded.push(face),
            Err(e) => eprintln!("[LiquidBounce] Failed to load theme font {name}: {e}"),
        }
    }
    let mut registry = target_registry()
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let previous = target_font_signature(&registry);
    for face in loaded {
        if registry
            .faces
            .get(&face.name)
            .is_none_or(|old| old.hash != face.hash)
        {
            registry.faces.insert(face.name.clone(), face);
        }
    }
    let primary = metadata["nativeFont"]
        .as_str()
        .filter(|name| registry.faces.contains_key(*name))
        .unwrap_or("Inter Medium")
        .to_owned();
    registry.primary = primary;
    let current = target_font_signature(&registry);
    if previous != current {
        registry.generation = registry.generation.wrapping_add(1);
    }
    Ok(())
}
pub fn liquidbounce_theme_font_info() -> serde_json::Value {
    let registry = target_registry()
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut faces = registry.faces.keys().cloned().collect::<Vec<_>>();
    faces.sort();
    serde_json::json!({"primary":registry.primary,"generation":registry.generation,"faces":faces})
}
fn target_font(bold: bool) -> (Arc<ab_glyph::FontVec>, bool) {
    let registry = target_registry()
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let face = &registry.faces[&registry.primary];
    // Preserve the included theme's established bold face; local faces derive bold.
    if bold && registry.primary == "Inter Medium" {
        if let Some(bold) = registry.faces.get("Inter Bold") {
            return (Arc::clone(&bold.font), false);
        }
    }
    (Arc::clone(&face.font), bold && !face.bold)
}
fn target_fallback(bold: bool, cjk: bool) -> Option<Arc<ab_glyph::FontVec>> {
    type Slot = std::sync::OnceLock<Option<Arc<ab_glyph::FontVec>>>;
    static COMMON: Slot = Slot::new();
    static COMMON_BOLD: Slot = Slot::new();
    static CJK: Slot = Slot::new();
    static CJK_BOLD: Slot = Slot::new();
    let slot = match (cjk, bold) {
        (false, false) => &COMMON,
        (false, true) => &COMMON_BOLD,
        (true, false) => &CJK,
        (true, true) => &CJK_BOLD,
    };
    slot.get_or_init(|| {
        let windows = std::env::var_os("WINDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("C:/Windows"));
        let name = match (cjk, bold) {
            (false, false) => "segoeui.ttf",
            (false, true) => "segoeuib.ttf",
            (true, false) => "msyh.ttc",
            (true, true) => "msyhbd.ttc",
        };
        let paths = if cfg!(target_os = "windows") {
            vec![windows.join("Fonts").join(name)]
        } else if cfg!(target_os = "macos") {
            vec![PathBuf::from(if cjk {
                "/System/Library/Fonts/PingFang.ttc"
            } else {
                "/System/Library/Fonts/Helvetica.ttc"
            })]
        } else {
            vec![PathBuf::from(if cjk {
                "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"
            } else if bold {
                "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"
            } else {
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"
            })]
        };
        paths.into_iter().find_map(|p| {
            std::fs::read(p)
                .ok()
                .and_then(|b| ab_glyph::FontVec::try_from_vec(b).ok())
                .map(Arc::new)
        })
    })
    .clone()
}
