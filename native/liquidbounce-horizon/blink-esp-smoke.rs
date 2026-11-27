use crate::blink_esp::{BlinkEsp,Player};
use serde_json::json;
fn player(first:bool,pose:i32)->Player{Player{first_person:first,width:0.6,height:if pose==5{1.5}else{1.8},pose,swim:0.4}}
#[test]fn modes_anchor_line_pose_and_first_person(){
 let positions=[[12.,70.,-8.],[13.,71.,-7.],[14.,72.,-6.]];let mut s=BlinkEsp::default();
 assert_eq!(s.mode,2);assert!(s.render.styles.iter().all(|v|v.get("Fill")==1.));
 for mode in 0..5{s.mode=mode;let v=s.visual(&positions,[70.,30.],player(true,0));assert!(v.body.quads.is_empty());assert_eq!(v.line.lines.len(),2);assert!(v.model.is_none());assert_eq!(v.ring.is_some(),mode==3);}
 s.mode=0;let b=s.visual(&positions,[0.,0.],player(false,5));assert_eq!(b.body.quads.len(),6);let ys=b.body.quads.iter().flatten().map(|p|p.position[1]).collect::<Vec<_>>();assert!((ys.iter().copied().fold(f64::INFINITY,f64::min)-69.95).abs()<1e-6);assert!((ys.iter().copied().fold(f64::NEG_INFINITY,f64::max)-71.55).abs()<1e-6);
 s.mode=1;assert_eq!(s.visual(&positions,[0.,0.],player(false,0)).model,Some(positions[0]));
 s.mode=2;let standing=s.visual(&positions,[0.,0.],player(false,0));assert_eq!((standing.body.quads.len(),standing.body.lines.len()),(36,72));for pose in[3,5]{assert_ne!(standing.body,s.visual(&positions,[0.,0.],player(false,pose)).body);}assert_ne!(standing.body,s.visual(&positions,[90.,30.],player(false,0)).body);
 s.mode=3;let ring=s.visual(&positions,[0.,0.],player(true,0));assert!(ring.body.lines.is_empty());assert_eq!(ring.ring.unwrap().0,positions[0]);s.radius=-0.5;assert!(s.visual(&positions,[0.,0.],player(true,0)).ring.is_none());s.line=0;assert!(s.visual(&positions,[0.,0.],player(true,0)).line.lines.is_empty());assert!(s.visual(&[],[0.,0.],player(false,0)).ring.is_none());
}
#[test]fn global_config_roundtrips_inactive_choices_and_solid_styles(){
 use crate::config::*;let mut s=BlinkEsp::default();s.mode=1;s.radius=1.25;s.box_color=0x12345678;s.light_percent=27;s.render.shader=true;s.render.style=8;s.render.styles[0].values.insert("Fill".into(),0.72);
 for format in[JsonFormat::File,JsonFormat::Interop]{let group=serialize_global_blink(&s,format,&str::to_owned);let p=parse_global_blink(&json!({"name":"Settings","value":[group.clone()]})).unwrap();assert!(p.issues.is_empty(),"{:?}",p.issues);let mut restored=BlinkEsp::default();for c in p.changes{c.apply(&mut restored);}assert_eq!(serialize_global_blink(&restored,format,&str::to_owned),group);}
 let before=serialize_global_blink(&s,JsonFormat::Interop,&str::to_owned);s.radius=1.75;let after=serialize_global_blink(&s,JsonFormat::Interop,&str::to_owned);let changes=blink_changed_values(&before,&after);assert_eq!(changes.len(),1);assert_eq!(changes[0]["name"],"Radius");s.mode=3;assert!(blink_changed_values(&after,&serialize_global_blink(&s,JsonFormat::Interop,&str::to_owned)).iter().any(|v|v["name"]=="Esp"));
}
#[test]fn runtime_uses_shared_queue_and_actual_server_rotation(){
 use crate::{runtime::LiquidBounceHooks,mod_api::*};let hooks=LiquidBounceHooks::default();let host=TestHost;let mut output=String::new();let mut e=ClientBlinkEspEvent{input:"0.4",geometry:&mut output};hooks.blink_esp(&host,&mut e);let a:serde_json::Value=serde_json::from_str(&output).unwrap();assert_eq!(a["line"]["lines"].as_array().unwrap().len(),1);assert_eq!(a["model"]["position"],json!([12.,70.,-8.]));assert_eq!(a["model"]["rotation"],json!([134.,-27.]));assert_eq!(a["model"]["lightPercent"],60);
}
#[test]fn radial_shader_and_actual_model_source_bridge(){
 let source=include_str!("blink_esp.wgsl");let module=naga::front::wgsl::parse_str(source).unwrap();naga::valid::Validator::new(naga::valid::ValidationFlags::all(),naga::valid::Capabilities::all()).validate(&module).unwrap();
 let model=include_str!("blink_native.rs");for fragment in["let mut snapshot=source.clone();","snapshot.previous=snapshot.position;","snapshot.previous_body_yaw=rotation[0];","snapshot.previous_head_pitch=rotation[1];","entity_light_block(original_position,","self.player_armor_layers()","self.player_cape_geometry(m, chest, camera, 0., packed, out)","self.held_item_geometry(m, camera, 0., packed, true, None, out)"]{assert!(model.contains(fragment),"{fragment}");}assert!(!model.contains("self.mobs.push"));assert!(!model.contains("alpha as f32"));
 let light=|n:u8,percent:f32|((f32::from(n)*percent*0.01).round()as i32).clamp(0,15);assert_eq!((light(15,60.),light(7,60.)),(9,4));
}
