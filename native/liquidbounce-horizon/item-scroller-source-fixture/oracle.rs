use crate::{item_scroller::ItemScroller,random::ThreadRandom};
#[test]
fn unchanged_kotlin_module_and_chronometer_match_guards_time_and_random_consumption() {
    let oracle:serde_json::Value=serde_json::from_str(include_str!("source-oracle.json")).unwrap();
    let rows=oracle["rows"].as_array().unwrap();
    assert_eq!(rows.len(),72);
    for row in rows {
        let input=&row["input"];
        let state=ItemScroller { delay:[input["from"].as_i64().unwrap() as i32,input["to"].as_i64().unwrap() as i32],last_update:row["last"].as_i64().unwrap(),..Default::default() };
        let mut random=ThreadRandom::seeded(123456789,row["thread"].as_u64().unwrap());
        let actual=state.can_perform_scroll(input["running"].as_bool().unwrap(),input["leftShift"].as_bool().unwrap()||input["rightShift"].as_bool().unwrap(),input["leftMouse"].as_bool().unwrap(),|from,to|random.next_i32_inclusive(from,to),||row["now"].as_i64().unwrap());
        assert_eq!(actual,row["allowed"].as_bool().unwrap(),"{row}");
        assert_eq!(i64::from(random.next_i32()),row["next"].as_i64().unwrap(),"sample order/count: {row}");
    }
    assert_eq!(oracle["click"],serde_json::json!([true,17,1,"QUICK_MOVE"]));
    assert_eq!(oracle["resetInBounds"],true);
}
#[test]
fn exact_elapsed_boundary_wrapping_and_queries_do_not_reset_the_timer() {
    use std::cell::Cell;
    let mut state=ItemScroller::default();state.reset_chronometer(500);
    let sampled=Cell::new(0);
    for (now,expected) in [(599,false),(600,true),(650,true)] {
        assert_eq!(state.can_perform_scroll(true,true,true,|from,to|{assert_eq!([from,to],[2,3]);sampled.set(sampled.get()+1);2},||now),expected);
        assert_eq!(state.last_update,500);
    }
    assert_eq!(sampled.get(),3);
    state.last_update=i64::MAX-49;
    assert!(state.can_perform_scroll(true,true,true,|_,_|2,||i64::MIN+50));
}
