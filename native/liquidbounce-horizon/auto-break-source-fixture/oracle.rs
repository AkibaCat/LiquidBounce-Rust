use crate::auto_break::{AutoBreak, KeyQuery};
use crate::keymapping_clicks::ClickQueues;

#[test]
fn unchanged_module_and_actual_official_keymapping_agree() {
    let rows: serde_json::Value = serde_json::from_str(include_str!("source-vectors.json")).unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 960);
    for row in rows.as_array().unwrap() {
        let input = &row["input"];
        let boolean = |name: &str| input[name].as_bool().unwrap();
        let query = KeyQuery {
            attack: boolean("attack"), miss_time: input["missTime"].as_i64().unwrap() as i32,
            block_hit: input["hit"] == "block",
            block_state_available: input["state"] != "missing", air: input["state"] == "air",
            destroying: boolean("destroying"),
        };
        let effect = AutoBreak.query(boolean("running"), query, boolean("original"));
        assert_eq!(effect.pressed, row["pressed"].as_bool().unwrap(), "{input}");
        assert_eq!(effect.click, row["click"].as_bool().unwrap(), "{input}");
        let mut queue = ClickQueues::default();
        if effect.click { queue.click("key.mouse.left", ["key.attack", "key.use", "key.forward", "key.screenshot"]); }
        for (index, mapping) in ["key.attack", "key.use", "key.forward", "key.screenshot"].iter().enumerate() {
            assert_eq!(i64::from(queue.pending(mapping)),row["counts"][index].as_i64().unwrap(),"{input}");
            let expected_held = boolean("original") && index == usize::from(!boolean("attack"));
            assert_eq!(row["held"][index].as_bool().unwrap(),expected_held,"click changed held state: {input}");
        }
    }
}

#[test]
fn actual_official_click_queue_lifecycle_and_rebinding_agree() {
    use std::collections::BTreeMap;
    let rows:serde_json::Value=serde_json::from_str(include_str!("click-vectors.json")).unwrap();
    let mut queue=ClickQueues::default();
    let names=["key.attack","key.use","key.forward","key.screenshot","key.inventory","key.hotbar.1","key.sneak"];
    let mut binds:BTreeMap<_,_>=names.iter().map(|name|(*name,"key.mouse.left".to_string())).collect();
    let mut down:BTreeMap<_,_>=names.iter().map(|name|(*name,false)).collect();
    for row in rows.as_array().unwrap() {
        let name=row["mapping"].as_str().unwrap();let key=row["key"].as_str().unwrap();
        match row["operation"].as_str().unwrap() {
            "click"=>queue.click(key,binds.iter().filter(|(_,bound)|bound.as_str()==key).map(|(name,_)|*name)),
            "bind"=>*binds.get_mut(name).unwrap()=key.to_owned(),
            "held"=>*down.get_mut(name).unwrap()=row["held"].as_bool().unwrap(),
            "consume"=>assert_eq!(queue.consume(name),row["consumed"].as_bool().unwrap(),"{row}"),
            "release"=>{queue.release_all();down.values_mut().for_each(|held|*held=false);},
            other=>panic!("unknown operation {other}"),
        }
        for name in names {
            assert_eq!(i64::from(queue.pending(name)),row["counts"][name].as_i64().unwrap(),"{row}");
            assert_eq!(down[name],row["down"][name].as_bool().unwrap(),"{row}");
        }
    }
}
