#[path="../../native/liquidbounce-behaviors/src/plugins.rs"]mod plugins;
use plugins::{Plugins,Effect};
#[test]fn command_timeouts_and_success_keep_original_order(){
 let mut p=Plugins::default();let mut id=||7;
 assert_eq!(p.start(0,&mut id),[Effect::Command("plugins")]);assert!(p.poll(4999,&mut id).is_empty());assert!(p.game_message(3000,"Plugins: not the expected line").is_empty());assert!(p.poll(5000,&mut id).is_empty());
 assert_eq!(p.poll(6000,&mut id),[Effect::Command("bukkit:plugins")]);assert_eq!(p.game_message(6001,"Plugins (3): Test"),[Effect::Disable]);assert!(!p.active());
 p.try_methods=0;assert_eq!(p.start(10,&mut id),[Effect::TimeOut,Effect::Disable]);
}
#[test]fn version_names_win_case_ties_and_prefix_fallback_uses_matching_ids(){
 let mut p=Plugins::default();p.try_methods=4;let mut n=10;let mut id=||{n+=1;n};
 assert_eq!(p.start(0,&mut id),[Effect::Suggest{id:11,input:"/version "}]);assert!(p.suggestions(100,12,vec!["Bad".into()],100,&mut id).is_empty());
 p.server_commands=vec!["essentials:home".into(),"GrimAC:grim".into(),"minecraft:help".into(),"help".into()];
 assert_eq!(p.suggestions(200,11,vec!["Essentials".into(),"grimac".into()],200,&mut id),[Effect::Found(vec!["Essentials".into(),"grimac".into(),"minecraft".into()]),Effect::Disable]);
 assert!(plugins::is_anticheat_plugin("GRIMAC"));assert!(!plugins::is_anticheat_plugin("Essentials"));
 p.server_commands.clear();p.start(1000,&mut id);assert_eq!(p.poll(2000,&mut id),[Effect::Suggest{id:13,input:"/bukkit:version "}]);assert_eq!(p.poll(3000,&mut id),[Effect::Suggest{id:14,input:"/"}]);
 assert_eq!(p.suggestions(3001,14,vec!["/WorldEdit:wand".into(),"worldedit:undo".into(),"plain".into()],3001,&mut id),[Effect::Found(vec!["WorldEdit".into()]),Effect::Disable]);
 p.disconnected();assert!(p.captured.is_none());assert!(p.server_commands.is_empty());assert!(!p.active());assert_eq!(plugins::TRY_TAGS.len(),3);
}
