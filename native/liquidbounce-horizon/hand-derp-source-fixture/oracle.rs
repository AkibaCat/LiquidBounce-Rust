#[test]
fn unchanged_kotlin_hand_derp_matches_all_modes_and_lifecycle() {
    use crate::hand_derp::{HandDerp,MainHand,Mode};
    for initial in [0,1] {
        let vectors:serde_json::Value=serde_json::from_str(if initial==0 { include_str!("source-vectors-0.json") } else { include_str!("source-vectors-1.json") }).unwrap();
        let mut state=HandDerp::default();let mut actual=MainHand::from_left(initial==0);state.initialize(actual);
        let mut enabled=false;let mut in_game=true;let mut no_swing=false;let mut ticket=0;
        for (row,line) in include_str!("commands.txt").lines().enumerate() {
            let p:Vec<_>=line.split_whitespace().collect();let mut sent=Vec::new();let mut cancelled=false;
            let mut emit=|hand:Option<MainHand>| { if let Some(hand)=hand { sent.push(hand.id()); } };
            let running=enabled&&in_game;
            match p[0] {
                "enabled"=>{ let value=p[1].parse::<bool>().unwrap();if enabled&&!value&&in_game { emit(state.on_disabled(actual)); } enabled=value; },
                "inGame"=>in_game=p[1].parse().unwrap(),
                "mode"=>state.set_mode(Mode::parse(p[1]).unwrap()),
                "delay"=>state.settings.delay=p[1].parse().unwrap(),
                "silent"=>state.settings.silent=p[1].parse().unwrap(),
                "actual"=>actual=MainHand::from_left(p[1]=="0"),
                "tick"=>{ state.begin_tick(running);if let Some(ticket)=state.ready_ticket() { emit(state.resume(ticket)); }emit(state.start_delay(running,|| { ticket+=1;ticket })); },
                "punch"=>if !no_swing { emit(state.punch(running)); },
                "noSwing"=>no_swing=p[1].parse().unwrap(),
                "metadata"=>cancelled=state.cancel_metadata(running,actual,p[2..].iter().map(|id|id.parse::<u8>().unwrap())),
                _=>panic!("{line}"),
            }
            assert_eq!(serde_json::json!([actual.id(),state.current().unwrap().id(),state.original().unwrap().id(),cancelled,sent]),vectors["rows"][row],"initial {initial}, row {row}: {line}");
        }
    }
}
