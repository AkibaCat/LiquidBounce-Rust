#[test]
fn unchanged_kotlin_derp_matches_every_scripted_tick() {
    use crate::derp::{Derp,PitchMode,Snapshot,YawMode};
    let fixture:serde_json::Value=serde_json::from_str(include_str!("source-vectors.json")).unwrap();
    let mut random=crate::random::ThreadRandom::seeded(fixture["seed"].as_u64().unwrap(),fixture["threadId"].as_u64().unwrap());
    let mut state=Derp::default();
    let mut enabled=true; let mut in_game=true; let mut row=0; let mut draws=0u64;
    for line in include_str!("commands.txt").lines() {
        let parts:Vec<_>=line.split_whitespace().collect();
        let float=|index:usize| parts[index].parse::<f32>().unwrap();
        let int=|index:usize| parts[index].parse::<i32>().unwrap();
        let boolean=|index:usize| parts[index].parse::<bool>().unwrap();
        match parts[0] {
            "yaw"=>state.set_yaw_mode(YawMode::parse(parts[1]).unwrap()),
            "pitch"=>state.settings.pitch_mode=PitchMode::parse(parts[1]).unwrap(),
            "floatYaw"=>match parts[1] { "Static"=>state.settings.yaw=float(3),"Offset"=>state.settings.yaw_offset=float(3),_=>panic!("unknown mode") },
            "intYaw"=>match parts[2] { "ForwardTicks"=>state.settings.forward_ticks=int(3),"BackwardTicks"=>state.settings.backward_ticks=int(3),"Speed"=>state.settings.spin_speed=int(3),_=>panic!("unknown setting") },
            "floatPitch"=>match parts[1] { "Static"=>state.settings.pitch=float(3),"Offset"=>state.settings.pitch_offset=float(3),_=>panic!("unknown mode") },
            "bool"=>match parts[1] { "SafePitch"=>state.settings.safe_pitch=boolean(2),"NotDuringSprint"=>state.settings.not_during_sprint=boolean(2),_=>panic!("unknown setting") },
            "enabled"=>{ let value=boolean(1); if enabled && !value && in_game { state.cancel_sequences(); } enabled=value; },
            "inGame"=>in_game=boolean(1),
            "tick"=>{
                let snapshot=Snapshot { yaw:float(1),pitch:float(2),sprint_key:boolean(3),sprinting:boolean(4) };
                let running=enabled&&in_game;
                state.begin_tick(running,snapshot);
                state.start_yaw_handler(running,snapshot);
                let actual=state.rotation_target(running,snapshot,&mut ||{ draws+=1; random.next_unit() }).unwrap();
                let expected=&fixture["rows"][row];
                match actual {
                    None=>assert!(expected.is_null(),"row {row}: {line}"),
                    Some(target)=>{
                        assert_eq!(target.rotation.yaw.to_bits(),expected[0].as_u64().unwrap() as u32,"row {row} yaw: {line}");
                        assert_eq!(target.rotation.pitch.to_bits(),expected[1].as_u64().unwrap() as u32,"row {row} pitch: {line}");
                        assert_eq!(draws,expected[2].as_u64().unwrap(),"row {row} draws: {line}");
                    }
                }
                row+=1;
            },
            _=>panic!("unknown command: {line}"),
        }
    }
    assert_eq!(row,fixture["rows"].as_array().unwrap().len());
}
