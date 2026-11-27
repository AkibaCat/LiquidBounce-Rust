// One focused test only. No shaders, GPU or frozen renderer matrix.
#[test]
fn active_theme_face_reload_and_native_atlas_invalidation() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let root = std::env::temp_dir().join(format!(
        "lb-theme-font-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(root.join("fonts")).unwrap();
    let metadata =
        |face: &str| serde_json::json!({"fonts":["not-the-face-name.ttf"],"nativeFont":face});
    std::fs::write(
        root.join("fonts/not-the-face-name.ttf"),
        include_bytes!("fixture-light.ttf"),
    )
    .unwrap();
    std::fs::write(
        root.join("metadata.json"),
        metadata("Inter Light").to_string(),
    )
    .unwrap();
    let font = font::Font::default();
    font.begin_frame();
    assert_eq!(
        font::liquidbounce_theme_font_info()["primary"],
        "Inter Medium"
    );
    let glyph = font.liquidbounce_target_glyph('W', false);
    assert!(glyph.rect.is_some());
    let old_pixels = font.first_patch_pixels();
    let old_epoch = font.epoch();
    let mut monitor = theme_font_runtime::ThemeFontMonitor::new(root.clone());
    assert_eq!(
        font::liquidbounce_theme_font_info()["primary"],
        "Inter Light"
    );
    let generation = font::liquidbounce_theme_font_info()["generation"].clone();
    font.begin_frame();
    assert_ne!(font.epoch(), old_epoch);
    assert_eq!(font.patch_count(), 0);
    let custom = font.liquidbounce_target_glyph('W', false);
    assert!(custom.rect.is_some());
    assert_ne!(font.first_patch_pixels(), old_pixels);
    font.liquidbounce_target_glyph('目', false);
    let count = font.patch_count();
    font.liquidbounce_target_glyph('W', false);
    assert_eq!(font.patch_count(), count);
    font.liquidbounce_target_glyph('W', true);
    assert!(font.patch_count() > count);
    monitor.reload();
    assert_eq!(
        font::liquidbounce_theme_font_info()["generation"],
        generation
    );
    let epoch = font.epoch();
    font.begin_frame();
    assert_eq!(font.epoch(), epoch);
    // Unknown nativeFont falls back to included nativeFont; matching is case sensitive.
    std::fs::write(
        root.join("metadata.json"),
        metadata("inter light").to_string(),
    )
    .unwrap();
    monitor.reload();
    assert_eq!(
        font::liquidbounce_theme_font_info()["primary"],
        "Inter Medium"
    );
    font.begin_frame();
    assert_eq!(font.patch_count(), 0);
    // Loaded faces remain registered, exactly as FontManager.fontFaces does.
    std::fs::write(root.join("fonts/not-the-face-name.ttf"), b"invalid font").unwrap();
    std::fs::write(
        root.join("metadata.json"),
        metadata("Inter Light").to_string(),
    )
    .unwrap();
    monitor.reload();
    assert_eq!(
        font::liquidbounce_theme_font_info()["primary"],
        "Inter Light"
    );
    font.begin_frame();
    let info = font::liquidbounce_theme_font_info();
    std::fs::write(root.join("metadata.json"), b"not json").unwrap();
    assert!(font::liquidbounce_load_theme_fonts(&root).is_err());
    assert_eq!(font::liquidbounce_theme_font_info(), info);
    println!(
        "Actual face name, fallback, custom glyph raster, cache epoch and reload checks passed"
    );
}
