use super::*;
use high_jump::{HighJump,Mode,TickContext};
use serde_json::Value;
#[test]fn unchanged_original_module_and_actual_coroutine_ticker_match_all_traces(){
    let inputs:Value=serde_json::from_str(include_str!("inputs.json")).unwrap();
    let oracle:Value=serde_json::from_str(include_str!("oracle.json")).unwrap();
    for (case,(actions,expected))in inputs.as_array().unwrap().iter().zip(oracle.as_array().unwrap()).enumerate(){
        let mut state=HighJump::default();let mut running=true;let mut velocity=-0.0;let mut waiters=vec![];
        for(step,(action,expected))in actions.as_array().unwrap().iter().zip(expected.as_array().unwrap()).enumerate(){
            let mut motion=None;
            match action["op"].as_str().unwrap(){
                "mode"=>state.set_mode(Mode::parse(action["value"].as_str().unwrap()).unwrap()),
                "running"=>{let next=action["value"].as_bool().unwrap();if running&&!next{state.disabled();}running=next;},
                "glide"=>state.settings.glide=action["value"].as_bool().unwrap(),
                "motion"=>state.settings.motion=f32::from_bits(action["bits"].as_str().unwrap().parse().unwrap()),
                "velocity"=>velocity=f64::from_bits(action["bits"].as_str().unwrap().parse().unwrap()),
                "jump"=>{let mut v=0.42;if let Some(id)=state.jump(running,&mut v){waiters.push(id);}motion=Some(v);},
                "tick"=>{
                    waiters.retain(|id|{let(keep,y)=state.advance_callback(*id,running);if let Some(y)=y{velocity=y;}keep});
                    if let Some(y)=state.game_tick(running,TickContext{on_ground:action["ground"].as_bool().unwrap(),fall_distance:f64::from_bits(action["fall"].as_str().unwrap().parse().unwrap()),tick_count:action["count"].as_i64().unwrap()as i32}){velocity=y;}
                },
                op=>panic!("unknown {op}"),
            }
            assert_eq!(velocity.to_bits().to_string(),expected["velocity"].as_str().unwrap(),"original velocity case={case},step={step},action={action}");
            assert_eq!(state.should_glide,expected["shouldGlide"].as_bool().unwrap(),"original glide case={case},step={step}");
            if let Some(m)=motion{assert_eq!(m.to_bits().to_string(),expected["motion"].as_str().unwrap(),"original jump case={case},step={step}");}
        }
    }
    assert_eq!(inputs.as_array().unwrap().len(),48);
}
#[test]fn parallel_waiters_complete_independently_and_glide_state_survives_lifecycle(){
    let mut s=HighJump::default();s.set_mode(Mode::Vulcan);let mut power=0.42;
    let a=s.jump(true,&mut power).unwrap();let b=s.jump(true,&mut power).unwrap();assert_ne!(a,b);
    for _ in 0..99{assert_eq!(s.advance_callback(a,true),(true,None));assert_eq!(s.advance_callback(b,true),(true,None));}
    assert_eq!(s.advance_callback(a,true),(false,Some(0.0)));assert_eq!(s.pending(),1);
    s.disabled();assert!(s.should_glide);s.set_mode(Mode::Vanilla);s.set_mode(Mode::Vulcan);assert!(s.should_glide);assert_eq!(s.pending(),0);
}
#[test]fn all_case_sensitive_mode_names_and_alias_are_preserved(){assert_eq!(Mode::parse("Vulcan"),Some(Mode::Vulcan));assert_eq!(Mode::parse("vulcan"),None);assert_eq!(Mode::Vulcan.tag(),"Vulcan-2.7.5");}
