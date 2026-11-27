#[path="../liquidbounce-behaviors/src/xray.rs"] mod xray;
#[test]
fn settings_and_geometry_policy(){
 let mut s=xray::Settings::default();
 assert!(s.full_bright&&!s.exposed_only&&s.background_opacity==0);
 assert!(s.blocks.contains("minecraft:waxed_oxidized_copper_chest"));
 assert!(s.blocks.contains("minecraft:light_blue_shulker_box"));
 assert!(s.render(true,[true;6]));s.exposed_only=true;
 assert!(!s.render(true,[true;6]));assert!(s.render(true,[false,true,true,true,true,true]));
 s.background_opacity=127;assert!(s.background(false,false));assert!(!s.background(true,false));assert!(!s.background(false,true));
 assert!(s.face(true,false,true,true));assert!(!s.face(true,false,false,false));assert!(!s.face(true,true,false,true));
 assert!(s.draw_side(false,true,true,false,true,true));assert!(!s.draw_side(false,true,true,true,true,true));
 s.reset_blocks();assert!(!s.blocks.contains("minecraft:copper_block"));assert!(s.blocks.contains("minecraft:diamond_ore"));
}
