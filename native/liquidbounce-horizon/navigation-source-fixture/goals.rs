use crate::navigation::{destination::{BlockDestination,Destination,NearDestination,StageDestination},geometry::{BlockPos,BlockRegion,Vec3}};
fn vec_raw(v:Vec3)->String{format!("{},{},{}",v.x.to_bits(),v.y.to_bits(),v.z.to_bits())}
fn region_raw(r:BlockRegion)->String{format!("{},{},{},{},{},{}",r.min.x,r.min.y,r.min.z,r.max.x,r.max.y,r.max.z)}
fn points(raw:&str)->Vec<BlockPos>{if raw=="-"{Vec::new()}else{raw.split(';').map(|raw|{let mut p=raw.split(',').map(|v|v.parse().unwrap());BlockPos::new(p.next().unwrap(),p.next().unwrap(),p.next().unwrap())}).collect()}}
#[test]fn unchanged_agent_destinations_and_geometry_oracle(){
 let root=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));let commands=std::fs::read_to_string(root.join("commands.txt")).unwrap();let expected=std::fs::read_to_string(root.join("results.txt")).unwrap();assert_eq!(commands.lines().count(),expected.lines().count());
 for(index,(command,expected))in commands.lines().zip(expected.lines()).enumerate(){
  let p:Vec<_>=command.split(' ').collect();let d=|raw:&str|f64::from_bits(raw.parse().unwrap());let v=|i:usize|Vec3::new(d(p[i]),d(p[i+1]),d(p[i+2]));let b=|i:usize|BlockPos::new(p[i].parse().unwrap(),p[i+1].parse().unwrap(),p[i+2].parse().unwrap());
  let result=match p[0]{
   "block"=>{let point=b(1);let packed=point.packed();let un=BlockPos::from_packed(packed);format!("{packed},{},{},{},{}",un.x,un.y,un.z,point.distance_sqr_long(b(4)))},
   "vec"=>{let a=v(1);let c=v(4);let contained=BlockPos::containing(a);format!("{}|{}|{}|{},{},{}",a==c,a.distance_sqr(c).to_bits(),vec_raw(a.fma(d(p[7]),c)),contained.x,contained.y,contained.z)},
   "near"=>{let goal=NearDestination::new(v(1),d(p[4]));format!("{}|{}",goal.is_goal(b(5)),goal.distance(b(5)).to_bits())},
   "blocks"=>{let goal=BlockDestination::new(points(p[1]));format!("{}|{}|{}",goal.is_goal(b(2)),goal.distance(b(2)).to_bits(),goal.ends().into_iter().map(vec_raw).collect::<Vec<_>>().join(";"))},
   "region"=>{let ends=if p[6]=="-"{Vec::new()}else{p[6].split(';').map(|raw|{let mut a=raw.split(',').map(d);Vec3::new(a.next().unwrap(),a.next().unwrap(),a.next().unwrap())}).collect()};region_raw(BlockRegion::around(v(1),&ends,d(p[4]),p[5].parse().unwrap()))},
   "stage"=>{let near=NearDestination::new(v(1),d(p[4]));let start=v(5);let range=d(p[8]);let unknown=points(p[12]);let border=d(p[13]);let level_known=|pos:BlockPos|!unknown.contains(&pos);let known=|pos:BlockPos|level_known(pos)&&(f64::from(pos.x)+0.5).abs()<=border&&(f64::from(pos.z)+0.5).abs()<=border;let goal=StageDestination::new(&near,start,range,known);format!("{}|{}|{}",goal.is_goal(b(9)),goal.distance(b(9)).to_bits(),StageDestination::<NearDestination,fn(BlockPos)->bool>::is_needed(start,near.center,range,level_known))},
   _=>panic!("{command}"),
  };assert_eq!(result,expected,"case {index}: {command}");
 }
}
