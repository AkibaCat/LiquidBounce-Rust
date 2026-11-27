#[test]
fn logoff_spot_native_lifecycle_settings_chat_and_packets() {
    use crate::{game_fixture::*,mobs::*,packets::PacketListener,mod_api::ServerboundPlayPacket as S,config,localization,lan,packets};
    let mut game=Game::default();
    let mut source=Mob::new(22,game.res.player_type().clone().unwrap(),Position{x:-0.2,y:64.8,z:8.},93.);
    source.join=Join(1234);source.head_yaw=18.;source.head_pitch=-30.;source.health=17.;source.skin=Some("actual-skin".into());source.cape=Some("actual-cape".into());source.slim=true;source.extra.main=Some(Stack("diamond_sword"));source.extra.armor[1]=Some(Stack("chestplate"));source.extra.chest=Some("chestplate".into());
    game.mobs.push(source);game.persist.uuids.insert(22,[0,0,0,42]);
    let r=game.remote.as_mut().unwrap();r.level_players.insert(22,42);r.level_player_names.insert(22,"Alice".into());r.level_player_textures.insert(22,"profile-textures".into());
    game.client_logoff_capture(&[22]);game.client_logoff_capture(&[22]);
    assert_eq!(game.client_logoff_spots.len(),1);assert_eq!(game.mobs.len(),2);assert_eq!(game.chat,vec!["Alice disappeared at X: -1, Y: 64, Z: 8."]);
    let id=game.client_logoff_spots[&42].id;
    let fake=game.mobs.iter().find(|m|m.id==id).unwrap();assert!((id as i32)<0);assert_eq!(fake.health,7.);assert_eq!(fake.skin.as_deref(),Some("actual-skin"));assert!(fake.slim);assert_eq!(fake.cape.as_deref(),Some("actual-cape"));assert_eq!(fake.head_yaw,18.);assert_eq!(fake.head_pitch,-30.);assert_eq!(fake.extra.main,Some(Stack("diamond_sword")));assert_eq!(fake.extra.armor[1],Some(Stack("chestplate")));assert_ne!(lan::playerdata::ints_uuid(game.persist.uuids[&id]),42);
    let service=game.hooks.state().behaviors.borrow().logoff_spot.clone();
    for mut packet in [S::Attack(crate::mod_api::Target{id:id as i32}),S::Interact(crate::mod_api::Target{id:id as i32}),S::SpectatorAction(crate::mod_api::Spectator{target:Some(id as i32)})]{let mut event=packets::PacketEvent::new(&mut packet);service.outgoing(&mut event);assert!(event.cancelled);}
    let mut packet=S::Attack(crate::mod_api::Target{id:333});let mut event=packets::PacketEvent::new(&mut packet);service.outgoing(&mut event);assert!(!event.cancelled);
    game.mobs.retain(|m|m.id!=22);game.persist.uuids.remove(&22);let r=game.remote.as_mut().unwrap();r.level_players.remove(&22);r.level_player_names.remove(&22);
    game.client_logoff_tick();assert_eq!(game.client_logoff_spots.len(),1);
    game.client_logoff_capture(&[id as i32]);assert_eq!(game.client_logoff_spots.len(),1);
    let mut back=game.mobs.iter().find(|m|m.id==id).unwrap().clone();back.id=23;back.join=Join(1235);game.mobs.push(back);game.persist.uuids.insert(23,[0,0,0,42]);game.remote.as_mut().unwrap().level_players.insert(23,42);game.remote.as_mut().unwrap().level_player_names.insert(23,"Alice".into());
    game.world.loaded=false;game.client_logoff_tick();assert!(game.client_logoff_spots.is_empty());assert_eq!(game.chat.last().unwrap(),"Alice position was unloaded and is no longer visible.");assert!(!service.is_fake(id as i32));assert!(!game.persist.uuids.contains_key(&id));
    game.world.loaded=true;game.client_logoff_capture(&[23]);let second=game.client_logoff_spots[&42].id;game.client_logoff_tick();assert!(game.client_logoff_spots.is_empty());assert_eq!(game.chat.last().unwrap(),"Alice reappeared.");assert!(!game.mobs.iter().any(|m|m.id==second));
    service.set_send_in_chat(false);let count=game.chat.len();game.client_logoff_capture(&[23]);assert_eq!(game.chat.len(),count);let hooks=game.hooks.clone();hooks.logoff_spot_disabled(&mut game);assert!(game.client_logoff_spots.is_empty());
    game.blink=Some(23);game.client_logoff_capture(&[23]);assert!(game.client_logoff_spots.is_empty());game.blink=None;game.window=false;game.client_logoff_capture(&[23]);assert!(game.client_logoff_spots.is_empty());
    let c=config::CommonSettings{enabled:true,hidden:false,key_bind:serde_json::json!({"key":"key.keyboard.unknown","action":"Toggle"})};let tree=config::serialize_logoff_spot(true,&c,config::JsonFormat::Interop,&str::to_owned).unwrap();let n=config::nodes(&tree,"LogoffSpot","LogoffSpot").unwrap();assert_eq!(n["SendInChat"]["value"],true);let input=serde_json::json!({"name":"LogoffSpot","value":[{"name":"SendInChat","value":"false"}]});let mut patch=config::SettingsPatch::default();config::parse_logoff_for_test(&mut patch,&config::nodes(&input,"LogoffSpot","LogoffSpot").unwrap());assert!(patch.issues.is_empty());assert!(matches!(patch.changes[0],config::SettingChange::LogoffSpot(config::LogoffSpotSetting(false))));
    for language in ["zh_cn","zh_tw"]{for key in ["liquidbounce.module.logoffSpot.description","liquidbounce.module.logoffSpot.sendInChat.description","liquidbounce.module.logoffSpot.messages.disappeared","liquidbounce.module.logoffSpot.messages.reappeared","liquidbounce.module.logoffSpot.messages.unloaded"]{assert_ne!(localization::describe(language,key),key);}}
}
