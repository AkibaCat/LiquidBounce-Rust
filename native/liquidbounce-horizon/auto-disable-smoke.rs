#[path="../../native/liquidbounce-behaviors/src/auto_disable.rs"]mod auto_disable;
use auto_disable::{AutoDisable,Reason};
#[test]fn trigger_and_reference_membership_survive_separate_config_loading(){
 let mut module=AutoDisable::default();assert_eq!(module.selected(),["Fly","Speed","NoClip","KillAura"]);
 assert!(module.handles(true,Reason::Flag));assert!(module.handles(true,Reason::Death));assert!(!module.handles(false,Reason::WorldChange));
 module.on=2;assert!(!module.handles(true,Reason::Flag));assert!(module.handles(true,Reason::Death));assert!(!module.handles(true,Reason::Disconnect));
 module.module_names=["Sprint".into()].into_iter().collect();assert!(!module.selected().iter().any(|v|v=="Sprint"));
 assert!(module.add("Sprint"));assert!(!module.add("Sprint"));assert!(module.remove("NoClip"));assert!(!module.remove("NoClip"));
 module.clear();assert!(module.selected().is_empty());assert!(module.module_names.is_empty());assert_eq!(Reason::Death.description(),"your death");assert_eq!(Reason::TAGS.len(),4);
}
