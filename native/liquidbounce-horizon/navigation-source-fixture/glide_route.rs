use crate::navigation::{glide::{abilities::GlideAbilities,route_search::{self,Route,RouteSearch}},search::{Failure,Outcome}};
fn glide_outcome(o:Outcome<Route>)->String{match o{
 Outcome::Searching=>"S".into(),Outcome::AlreadyThere=>"A".into(),Outcome::Failed(f)=>format!("E:{}",match f{Failure::InvalidDestination=>"INVALID_DESTINATION",Failure::TooFar=>"TOO_FAR",Failure::NoPath=>"NO_PATH",Failure::TooComplex=>"TOO_COMPLEX",Failure::TimedOut=>"TIMED_OUT"}),
 Outcome::Found(r)=>format!("F:{}:{}:{}:{}:{}",vector(r.start),r.points.into_iter().map(vector).collect::<Vec<_>>().join(";"),vector(r.landing),r.rockets,r.approach.to_bits())}}
type GlideSearch=RouteSearch<S,World>;
fn extra(a:&[&str],t:&mut Terrain<S,World>,search:&mut Option<GlideSearch>)->String{match a[0]{
 "region"=>{let r=route_search::region(v(a[1]),v(a[2]));format!("{},{},{},{},{},{}",r.min.x,r.min.y,r.min.z,r.max.x,r.max.y,r.max.z)},
 "glide"=>{let s=RouteSearch::new(Terrain::new(t.level().clone(),t.width(),t.height(),t.can_open_doors()),v(a[1]),v(a[2]),d(a[3]),GlideAbilities{rockets:a[4].parse().unwrap(),rocket_ticks:a[5].parse().unwrap(),glide_ticks:a[6].parse().unwrap()});let state=s.fixture_state();*search=Some(s);state},
 "advance"=>{let s=search.as_mut().unwrap();format!("{}#{}",glide_outcome(s.advance()),s.fixture_state())},
 "preview"=>search.as_ref().unwrap().preview().into_iter().map(vector).collect::<Vec<_>>().join(";"),_=>panic!("{a:?}")}}
