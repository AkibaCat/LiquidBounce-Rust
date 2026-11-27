use super::Passage;
use crate::navigation::{flight::FlyRoute,glide::rockets::Rockets};
fn passage_state(p:&Passage)->String {format!("{}^{}^{}^{}^{}^{}^{}^{}^{}^{}^{}^{}",p.points.iter().copied().map(vector).collect::<Vec<_>>().join(";"),p.arcs.iter().map(|v|v.to_bits().to_string()).collect::<Vec<_>>().join(","),p.end_arc.to_bits(),p.segment,p.last_x_rot.to_bits(),p.reaches_target,p.passes_target,p.drift.to_bits(),p.remaining.to_bits(),p.rockets.left(),p.rockets.boost_ticks(),p.rockets.can_fire())}
fn extra(a:&[&str],t:&mut Terrain<S,World>,passage:&mut Option<Passage>)->String {match a[0]{
 "passage"=>{let p=Passage::new(v(a[1]),Rockets::new(a[2].parse().unwrap(),a[3].parse().unwrap()),d(a[4]),a[5].parse().unwrap());let result=passage_state(&p);*passage=Some(p);result},
 "follow"=>{let p=passage.as_mut().unwrap();let points:Vec<_>=a[1].split(';').map(v).collect();p.follow(&FlyRoute{start:points[0],points,can_reach:a[2].parse().unwrap()},(a[3]!="-").then(||a[3].parse().unwrap()));passage_state(p)},
 "lands"=>{let p=passage.as_mut().unwrap();p.lands_at_target=a[1].parse().unwrap();passage_state(p)},
 "steer"=>{let p=passage.as_mut().unwrap();let c=p.steer_with(||{t.level().events.borrow_mut().push("T".into());Terrain::new(t.level().clone(),t.width(),t.height(),t.can_open_doors())},v(a[1]),v(a[2]));format!("{},{},{},{}#{}",c.y_rot.to_bits(),c.x_rot.to_bits(),c.should_fire_rocket,c.is_boosted,passage_state(p))},
 "project"=>{let p=passage.as_ref().unwrap().project(v(a[1]),a[2].parse().unwrap());format!("{}:{}:{}:{}",p.segment,p.arc.to_bits(),vector(p.point),p.distance.to_bits())},
 "look"=>{let p=passage.as_ref().unwrap();vector(p.look_ahead(p.project(v(a[1]),a[2].parse().unwrap()),d(a[3])))},
 "rollout"=>{let p=passage.as_ref().unwrap();p.rollout(&mut Terrain::new(t.level().clone(),t.width(),t.height(),t.can_open_doors()),v(a[1]),v(a[2]),p.project(v(a[1]),a[3].parse().unwrap()),f32::from_bits(a[4].parse().unwrap()),f32::from_bits(a[5].parse().unwrap())).to_bits().to_string()},_=>panic!("{a:?}")}}
