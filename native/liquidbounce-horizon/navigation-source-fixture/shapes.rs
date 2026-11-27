use crate::navigation::{aabb::{Aabb,Axis,Clip,Direction},geometry::Vec3,voxel_shape::VoxelShape};
fn d(raw:&str)->f64{f64::from_bits(raw.parse().unwrap())}
fn values(raw:&str)->Vec<f64>{raw.split(',').map(d).collect()}
fn v(raw:&str)->Vec3{let a=values(raw);Vec3::new(a[0],a[1],a[2])}
fn b(raw:&str)->Aabb{let a=values(raw);Aabb::new(Vec3::new(a[0],a[1],a[2]),Vec3::new(a[3],a[4],a[5]))}
fn s(raw:&str)->VoxelShape{VoxelShape::new(if raw=="-"{Vec::new()}else{values(raw).as_chunks::<6>().0.to_vec()})}
fn vector(v:Vec3)->String{format!("{},{},{}",v.x.to_bits(),v.y.to_bits(),v.z.to_bits())}
fn bbox(b:Aabb)->String{format!("{},{}",vector(b.min),vector(b.max))}
fn raw(v:impl IntoIterator<Item=f64>)->String{v.into_iter().map(|v|v.to_bits().to_string()).collect::<Vec<_>>().join(",")}
fn face(v:Option<Direction>)->&'static str{match v{None=>"-",Some(Direction::Down)=>"DOWN",Some(Direction::Up)=>"UP",Some(Direction::North)=>"NORTH",Some(Direction::South)=>"SOUTH",Some(Direction::West)=>"WEST",Some(Direction::East)=>"EAST"}}
#[test]fn unchanged_agent_aabb_clip_and_voxel_collision_shapes(){
 let root=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));let commands=std::fs::read_to_string(root.join("commands.txt")).unwrap();let expected=std::fs::read_to_string(root.join("results.txt")).unwrap();assert_eq!(commands.lines().count(),expected.lines().count());
 for(index,(command,expected))in commands.lines().zip(expected.lines()).enumerate(){let p:Vec<_>=command.split(' ').collect();let result=match p[0]{
  "box"=>{let a=b(p[1]);let other=b(p[2]);let point=v(p[3]);let offset=v(p[4]);[bbox(a),vector(a.size()),vector(a.center()),vector(a.bottom_center()),bbox(a.moved(offset)),bbox(a.inflate(offset)),bbox(a.expand_towards(offset)),bbox(a.minmax(other)),bbox(a.intersect(other)),a.intersects(other).to_string(),a.contains(point).to_string(),a.distance_sqr(point).to_bits().to_string(),a.clip(point,offset).map_or_else(||"-".into(),vector),bbox(a.with_min_y(offset.y)),bbox(a.with_max_y(offset.y)),bbox(Aabb::of_size(point,offset)),bbox(Aabb::unit_cube_from_lower_corner(point))].join("|")},
  "shape"=>{let shape=s(p[1]);let axis=Axis::ALL[p[2].parse::<usize>().unwrap()];let body=b(p[3]);let distance=d(p[4]);let moved=shape.moved(v(p[5]));[shape.is_empty().to_string(),shape.box_count().to_string(),raw(shape.to_array()),shape.aabbs().iter().copied().map(bbox).collect::<Vec<_>>().join(";"),raw(Axis::ALL.map(|a|shape.min(a))),raw(Axis::ALL.map(|a|shape.max(a))),shape.bounds().map_or_else(||"-".into(),bbox),shape.collide(axis,body,distance).to_bits().to_string(),raw(moved.to_array()),shape.same_instance(&moved).to_string(),(shape==s(p[6])).to_string(),VoxelShape::collide_all(axis,body,[&shape,&moved],distance).to_bits().to_string()].join("|")},
  "ray"=>{let mut clip=Clip::new(v(p[1]),v(p[2]));if p[3]!="-"{for raw in p[3].split(';'){clip.include(b(raw));}}format!("{}|{}",face(clip.face),vector(clip.location()))},
  "identity"=>{let a=values(p[1]);let shape=VoxelShape::box_shape(a.try_into().unwrap());format!("{}|{}",shape.same_instance(&VoxelShape::block()),VoxelShape::empty().same_instance(&VoxelShape::empty().moved(Vec3::new(1.0,2.0,3.0))))},_=>panic!("{command}")};
  assert_eq!(result,expected,"case {index}: {command}");
 }
}
