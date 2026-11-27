#[test]fn complete_modes_motion_and_inactive_glide_roundtrip_raw_imports(){
    let common=CommonSettings{enabled:true,hidden:false,key_bind:json!({"key":"key.keyboard.unknown","action":"Toggle"})};
    for mode in [high_jump::Mode::Vanilla,high_jump::Mode::Vulcan]{for motion in [-3.75,-0.0,0.8,12.5]{for format in [JsonFormat::File,JsonFormat::Interop]{
        let mut original=high_jump::HighJump::default();original.settings.mode=mode;original.settings.motion=motion;original.settings.glide=true;
        let root=serialize_high_jump(&original,&common,format,&str::to_owned).unwrap();
        assert_eq!(root["value"].as_array().unwrap().iter().map(|v|v["name"].as_str().unwrap()).collect::<Vec<_>>(),if format==JsonFormat::File{vec!["Enabled","Bind","Hidden","Mode","Motion"]}else{vec!["Bind","Hidden","Mode","Motion"]});
        let n=nodes(&root,"HighJump","HighJump").unwrap();assert_eq!(n["Motion"]["range"],if format==JsonFormat::Interop{json!({"from":0.2,"to":10.0})}else{Value::Null});
        let mut patch=SettingsPatch::default();parse_high_jump(&mut patch,&n);assert!(patch.issues.is_empty(),"{:?}",patch.issues);
        let mut restored=high_jump::HighJump::default();for SettingChange::HighJump(v)in patch.changes{v.apply(&mut restored);}
        assert_eq!(restored.settings.mode,mode);assert_eq!(restored.settings.motion.to_bits(),motion.to_bits());assert!(restored.settings.glide);
    }}}
}
#[test]fn alias_mode_import_and_nonfinite_export_follow_shared_helpers(){
    let common=CommonSettings{enabled:false,hidden:false,key_bind:json!({})};let mut s=high_jump::HighJump::default();
    let mut tree=serialize_high_jump(&s,&common,JsonFormat::File,&str::to_owned).unwrap();
    let mode=tree["value"].as_array_mut().unwrap().iter_mut().find(|v|v["name"]=="Mode").unwrap();mode["active"]=json!("Vulcan");
    let mut patch=SettingsPatch::default();parse_high_jump(&mut patch,&nodes(&tree,"HighJump","HighJump").unwrap());assert!(patch.issues.is_empty());
    for SettingChange::HighJump(v)in patch.changes{v.apply(&mut s);}assert_eq!(s.settings.mode,high_jump::Mode::Vulcan);
    s.settings.motion=f32::NAN;assert!(serialize_high_jump(&s,&common,JsonFormat::File,&str::to_owned).is_err());
}
