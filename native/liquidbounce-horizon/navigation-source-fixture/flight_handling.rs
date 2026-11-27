use super::*;
use std::cell::RefCell;
struct Player {
    position: Vec3, delta: Vec3, tick: i32, yaw: f32, sprint: bool,
    horizontal: bool, vertical: bool, events: RefCell<Vec<&'static str>>,
}
impl FlightPlayer for Player {
    fn position(&self)->Vec3 {self.events.borrow_mut().push("p");self.position}
    fn delta_movement(&self)->Vec3 {self.events.borrow_mut().push("d");self.delta}
    fn tick_count(&self)->i32 {self.events.borrow_mut().push("t");self.tick}
    fn horizontal_collision(&self)->bool {self.events.borrow_mut().push("h");self.horizontal}
    fn vertical_collision(&self)->bool {self.events.borrow_mut().push("v");self.vertical}
    fn y_rot(&self)->f32 {self.events.borrow_mut().push("y");self.yaw}
    fn is_sprinting(&self)->bool {self.events.borrow_mut().push("s");self.sprint}
}
fn raw(value:f64)->String {if value.is_nan(){0x7ff8_0000_0000_0000_u64}else{value.to_bits()}.to_string()}
fn vec(value:Vec3)->String {[value.x,value.y,value.z].map(raw).join(",")}
fn acceleration(value:Acceleration)->&'static str {match value{Acceleration::Direct=>"DIRECT",Acceleration::Momentum=>"MOMENTUM"}}
fn state(h:&Handling)->String {
    let mut fields=vec![acceleration(h.control).to_owned()];
    fields.extend([h.base_speed,h.base_vertical_speed,h.measured_drag,h.measured_vertical_drag,h.measured_thrust,h.measured_sprint_thrust,h.measured_vertical_thrust].map(raw));
    fields.extend([h.last_position.map_or_else(||"-".into(),vec),h.last_move().map_or_else(||"-".into(),vec),h.last_tick.to_string(),h.last_pressed.map_or_else(||"-".into(),|p|p.java_hash().to_string())]);
    fields.extend([h.drag(),h.vertical_drag(),h.speed(),h.vertical_speed(),h.step()].map(raw));fields.push(h.coasts().to_string());fields.push(raw(h.coast_distance()));fields.join("|")
}
#[test]
fn unchanged_flight_handling_measurements_and_live_getter_order(){
    let root=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));let commands=std::fs::read_to_string(root.join("commands.txt")).unwrap();let expected=std::fs::read_to_string(root.join("results.txt")).unwrap();assert_eq!(commands.lines().count(),expected.lines().count());
    let mut handling=Handling::default();let mut player=Player{position:Vec3::ZERO,delta:Vec3::ZERO,tick:0,yaw:0.0,sprint:false,horizontal:false,vertical:false,events:RefCell::new(Vec::new())};let mut actual=Vec::new();
    for command in commands.lines(){
        let p:Vec<_>=command.split(' ').collect();let d=|s:&str|f64::from_bits(s.parse().unwrap());let v=|s:&str|{let a:Vec<_>=s.split(',').map(d).collect();Vec3::new(a[0],a[1],a[2])};let mut result="-".to_owned();
        match p[0]{
            "new"=>handling=Handling::default(),
            "base"=>{let base=FlyAbilities::new(if p[1]=="DIRECT"{Acceleration::Direct}else{Acceleration::Momentum},d(p[2]),d(p[3]),p[4].parse().unwrap());let value=handling.abilities(base);result=format!("{},{},{},{},{},{}",acceleration(value.control()),raw(value.horizontal_speed()),raw(value.vertical_speed()),value.is_flying(),raw(value.horizontal_cost()),raw(value.vertical_cost()));},
            "player"=>{player.tick=p[1].parse().unwrap();player.position=v(p[2]);player.delta=v(p[3]);player.yaw=f32::from_bits(p[4].parse().unwrap());player.sprint=p[5].parse().unwrap();player.horizontal=p[6].parse().unwrap();player.vertical=p[7].parse().unwrap();},
            "record"=>handling.record(&player,if p[1]=="-"{None}else{Some(Input::from_mask(p[1].parse().unwrap()))}),
            "rest"=>result=vec(handling.rest(&player)),
            _=>panic!("{command}"),
        }
        actual.push(format!("{result}|{}|{}",state(&handling),player.events.borrow().join(",")));player.events.borrow_mut().clear();
    }
    std::fs::write(root.join("actual.txt"),actual.join("\n")+"\n").unwrap();let failures:Vec<_>=commands.lines().zip(expected.lines()).zip(&actual).enumerate().filter(|(_,((_,expected),actual))|expected!=actual).map(|(i,((command,expected),actual))|format!("case {i}: {command}\nexpected {expected}\nactual   {actual}")).collect();assert!(failures.is_empty(),"{} mismatches\n{}",failures.len(),failures.iter().take(8).cloned().collect::<Vec<_>>().join("\n"));
}
