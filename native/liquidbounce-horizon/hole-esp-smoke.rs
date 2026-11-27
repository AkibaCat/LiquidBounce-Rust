use hole_tracker::{AIR,BLAST_RESISTANT,INDESTRUCTIBLE,BREAKABLE,Hole,HoleTracker,HoleManager,Region,Pos};
use std::collections::HashMap;
fn room(p:Pos,size:[i32;2],material:i8)->HashMap<Pos,i8>{
    let mut w=HashMap::new();
    for x in p[0]-1..=p[0]+size[0]{for z in p[2]-1..=p[2]+size[1]{
        w.insert([x,p[1]-1,z],material);
        if x<p[0]||x>=p[0]+size[0]||z<p[2]||z>=p[2]+size[1]{w.insert([x,p[1],z],material);}
    }}w
}
fn scan(w:&HashMap<Pos,i8>)->HoleTracker{let mut t=HoleTracker::default();t.scan(Region{min:[-4,0,-4],max:[12,16,12]},16,&mut|p|*w.get(&p).unwrap_or(&AIR));t}
#[test]fn safe_columns_all_sizes_and_invalidation(){
    assert_eq!([hole_tracker::classify(true,4e6),hole_tracker::classify(false,599.99),hole_tracker::classify(false,600.),hole_tracker::classify(false,3_600_000.)],[AIR,BREAKABLE,BLAST_RESISTANT,INDESTRUCTIBLE]);
    for(size,material,kind)in[([1,1],INDESTRUCTIBLE,0),([1,1],BLAST_RESISTANT,1),([2,1],BLAST_RESISTANT,2),([1,2],BLAST_RESISTANT,2),([2,2],BLAST_RESISTANT,3)]{
        let mut w=room([2,5,2],size,material);let mut t=scan(&w);assert_eq!(t.holes.len(),1);let h=*t.holes.values().next().unwrap();assert_eq!((h.position,h.size,h.kind),([2,5,2],size,kind));
        w.insert([2,7,2],BREAKABLE);t.record_block([2,7,2],BREAKABLE,16,&mut|p|*w.get(&p).unwrap_or(&AIR));assert!(t.holes.is_empty());
        w.remove(&[2,7,2]);t.record_block([2,7,2],AIR,16,&mut|p|*w.get(&p).unwrap_or(&AIR));assert_eq!(t.holes.len(),1);
        w.remove(&[1,5,2]);t.record_block([1,5,2],AIR,16,&mut|p|*w.get(&p).unwrap_or(&AIR));assert!(t.holes.is_empty());
    }
    let mut w=room([2,14,2],[1,1],INDESTRUCTIBLE);assert!(scan(&w).holes.is_empty());w=room([2,5,2],[1,1],INDESTRUCTIBLE);w.insert([1,5,2],BLAST_RESISTANT);assert_eq!(scan(&w).holes.values().next().unwrap().kind,1);
}
#[test]fn manager_subscribers_regions_and_four_block_movement(){
    let mut m=HoleManager::default();let mut read=|_|AIR;
    m.subscribe(7,4,4,[20,5,20],32,&mut read);let r=m.region;assert_eq!(r,Region::around([20,5,20],4,4));
    m.settings(7,8,8);m.player_post_tick([23,5,20],32,&mut read);assert_eq!(m.region,r);
    m.player_post_tick([22,6,21],32,&mut read);assert_eq!(m.region,Region::around([22,6,21],8,8));
    m.subscribe(8,12,10,[22,6,21],32,&mut read);assert_eq!(m.region,Region::around([22,6,21],8,8));
    m.player_post_tick([26,6,21],32,&mut read);assert_eq!(m.region,Region::around([26,6,21],12,10));
    let differences=Region::around([27,6,21],12,10).difference(m.region);assert_eq!(differences.len(),1);assert_eq!(differences[0].min[0],39);
    m.unsubscribe(7);assert!(m.active());m.unsubscribe(8);assert!(!m.active());assert_eq!(m.region,Region::default());assert!(m.tracker.holes.is_empty());
}
#[test]fn geometry_modes_gradient_fade_alpha_and_bounds(){
    let h=Hole{position:[0,0,0],size:[2,1],kind:2};let mut s=hole_esp::HoleEsp::default();s.colors[2]=0x0135bacc;
    let plane=s.geometry([0.,0.,0.],&[h]);assert_eq!(plane["quads"].as_array().unwrap().len(),5);assert_eq!(plane["lines"].as_array().unwrap().len(),4);assert_eq!(plane["quads"][0][0]["color"][3],serde_json::json!(50f32/255.));assert_eq!(plane["quads"][1][2]["color"][3],0.);
    assert_eq!(plane["quads"][1][2]["position"][1].as_f64().unwrap(),f64::from(0.7f32));
    s.glow_height=0.;s.plane_outline=false;let flat=s.geometry([0.,0.,0.],&[h]);assert_eq!(flat["quads"].as_array().unwrap().len(),1);assert!(flat["lines"].as_array().unwrap().is_empty());
    s.mode=0;let box_=s.geometry([0.,0.,0.],&[h]);assert_eq!(box_["quads"].as_array().unwrap().len(),6);assert_eq!(box_["lines"].as_array().unwrap().len(),12);
    assert_eq!(s.fade([0.,-7.,0.],&h),1.);assert_eq!(s.fade([0.,8.,0.],&h),0.);s.distance_fade=0.;assert_eq!(s.fade([0.,200.,0.],&h),1.);assert!(s.geometry([33.,0.,0.],&[h])["quads"].as_array().unwrap().is_empty());
    s.distance_fade=1.;s.horizontal=10;let fade=s.geometry([9.,0.,0.],&[h]);assert_eq!(fade["quads"][0][0]["color"][3],serde_json::json!(5f32/255.));assert_eq!(fade["lines"][0][0]["color"][3],serde_json::json!(10f32/255.));
}
#[test]fn full_file_and_interop_roundtrip_inactive_modes_and_descriptions(){
    use config::*;let mut s=hole_esp::HoleEsp::default();s.mode=0;s.box_outline=false;s.plane_outline=false;s.glow_height=0.42;s.horizontal=99;s.vertical=11;s.distance_fade=0.8;s.colors=[1,-1,i32::MIN,0x12345678];
    let common=CommonSettings{enabled:true,key_bind:serde_json::json!({"key":"key.keyboard.unknown","modifiers":[]}),hidden:false};
    for format in[JsonFormat::File,JsonFormat::Interop]{let tree=serialize_hole_esp(&s,&common,format,&str::to_owned).unwrap();let nodes=nodes(&tree,"HoleESP","HoleESP").unwrap();let mut patch=SettingsPatch::default();settings::parse_hole_esp(&mut patch,&nodes);assert!(patch.issues.is_empty());assert_eq!(patch.changes.len(),11);let mut restored=hole_esp::HoleEsp::default();for SettingChange::HoleEsp(change)in patch.changes{change.apply(&mut restored);}assert_eq!(s,restored);}
    let tree=serialize_hole_esp(&s,&common,JsonFormat::Interop,&str::to_owned).unwrap();
    fn check(v:&serde_json::Value,d:&serde_json::Value){if let Some(k)=v["key"].as_str().filter(|k|k.starts_with("liquidbounce.")){assert!(d["descriptions"][format!("{k}.description")].as_str().is_some(),"{k}");}match v{serde_json::Value::Array(a)=>for v in a{check(v,d)},serde_json::Value::Object(o)=>for v in o.values(){check(v,d)},_=>{}}}
    for source in[include_str!("localization/display_zh_cn.json"),include_str!("localization/display_zh_tw.json")]{check(&tree,&serde_json::from_str(source).unwrap());}
}
