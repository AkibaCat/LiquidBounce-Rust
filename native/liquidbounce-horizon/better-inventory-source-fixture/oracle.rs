use crate::better_inventory::*;
use serde_json::{Value,json};
fn i(v:&Value,k:&str)->i32 {v[k].as_i64().unwrap() as i32}
fn b(v:&Value,k:&str)->bool {v[k].as_bool().unwrap()}
fn f(v:&Value,k:&str)->f32 {f32::from_bits(i(v,k) as u32)}
fn bits(v:f32)->i32 {v.to_bits() as i32}
#[test]
fn every_jvm_locale_matches_formatter_and_only_fractional_seconds_localize() {
    let locales:Value=serde_json::from_str(include_str!("locale-oracle.json")).unwrap();
    let module=BetterInventory{cooldown_mode:CooldownMode::DurationSeconds,..Default::default()};
    for row in locales["rows"].as_array().unwrap() {
        let locale=row["locale"].as_str().unwrap();
        for (index,ticks) in [10,-10,0,i32::MIN].into_iter().enumerate() {
            let draw=module.cooldown_with_locale(true,false,true,0.5,Some((ticks,0)),0,0,locale).unwrap().unwrap();
            assert_eq!(draw.text,format!("{}s",row["formats"][index].as_str().unwrap()),"{locale} ticks={ticks}");
        }
        assert_eq!(module.cooldown_with_locale(true,false,true,0.5,Some((100,0)),0,0,locale).unwrap().unwrap().text,"5s");
    }
}
#[test]
fn unchanged_module_and_layout_differential() {
    let commands:Value=serde_json::from_str(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"),"/commands.json")).unwrap()).unwrap();
    let vectors:Value=serde_json::from_str(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"),"/source-vectors.json")).unwrap()).unwrap();
    for (index,row) in commands.as_array().unwrap().iter().enumerate() {
        let s=&row["state"];
        let module=BetterInventory {
            highlight_clicked:b(s,"highlight"),highlight_mode:HighlightMode::parse(s["highlightMode"].as_str().unwrap()).unwrap(),highlight_color:i(s,"highlightColor"),
            text_cooldown:b(s,"text"),cooldown_mode:match s["cooldownMode"].as_str().unwrap(){"PERCENTAGE"=>CooldownMode::Percentage,"DURATION_TICKS"=>CooldownMode::DurationTicks,_=>CooldownMode::DurationSeconds},
            cooldown_scale:f(s,"cooldownScale"),cooldown_color:i(s,"cooldownColor"),container_view:b(s,"container"),skip_empty_stack:b(s,"skipEmpty"),container_scale:f(s,"containerScale"),relative_to_mouse:b(s,"relative"),render_offset_x:f(s,"offsetX"),render_offset_y:f(s,"offsetY"),move_cursor:b(s,"move"),
        };
        let running=b(row,"enabled")&&b(row,"inGame");let mut ops=Vec::new();
        match module.highlight(running,i(row,"last"),i(row,"slot"),i(row,"x"),i(row,"y")) {
            Some(Highlight::Border{rects,color})=>for rect in rects {ops.push(json!({"op":"fill","rect":rect,"color":color}));},
            Some(Highlight::Texture{rect})=>ops.push(json!({"op":"sprite","rect":rect})),None=>{},
        }
        let entry=row["entry"].as_array().map(|e|(e[0].as_i64().unwrap() as i32,e[1].as_i64().unwrap() as i32));
        match module.cooldown(running,b(row,"empty"),b(row,"player"),f(row,"progress"),entry,i(row,"x"),i(row,"y")) {
            Ok(Some(text))=>ops.extend([json!({"op":"push"}),json!({"op":"scale","bits":bits(text.scale)}),json!({"op":"text","text":text.text,"x":text.center_x,"y":text.y,"color":text.color}),json!({"op":"pop"})]),
            Ok(None)=>{},Err(DrawError::MissingCooldownEntry)=>ops.extend([json!({"op":"push"}),json!({"op":"scale","bits":bits(module.cooldown_scale)}),json!({"op":"missingEntry"})]),Err(error)=>panic!("{error:?}"),
        }
        let items:Vec<_>=row["items"].as_array().unwrap().iter().map(|i|i.as_bool().unwrap()).collect();
        let mut preview=false;
        if BetterInventory::screen_preview_eligible(b(row,"carriedEmpty"),b(row,"hover")) {
            match module.container(running,b(row,"empty"),b(row,"component").then_some(&items),i(row,"left"),i(row,"top"),i(row,"mouseX"),i(row,"mouseY")) {
                Ok(Some(view))=>{preview=true;ops.push(json!({"op":"list","indices":view.indices,"centerBits":view.center.map(bits),"scaleBits":bits(view.scale),"dimensions":view.dimensions}));},
                Ok(None)=>{},Err(DrawError::InvalidContainerScale)=>ops.push(json!({"op":"invalidScale"})),Err(error)=>panic!("{error:?}"),
            }
        }
        assert_eq!(json!({"move":module.should_move_cursor(running),"preview":preview,"ops":ops}),vectors["rows"][index],"row {index}: {row}");
    }
}
