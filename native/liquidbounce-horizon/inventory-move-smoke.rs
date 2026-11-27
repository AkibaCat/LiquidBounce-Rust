#[test]
fn policies() {
    use inventory_move_model::*;
    let mut s=Settings::default();
    let own=Screen{open:true,container:true,inventory:true,editing:false};
    let chest=Screen{inventory:false,..own};
    assert!(s.handles(true,own,0));assert!(!s.handles(true,own,5));assert!(!s.handles(true,own,7));
    assert!(!s.handles(true,Screen{editing:true,..own},0));assert!(s.handles(true,Screen::default(),9));
    s.behavior=1;assert!(s.handles(true,own,0));assert!(!s.handles(true,chest,0));
    assert!(s.safe_click(true,own,0,[true,false,false,false,false,false]));
    assert!(!s.safe_click(true,own,2,[true;6]));assert!(!s.safe_click(true,chest,0,[true;6]));
    s.behavior=2;assert!(!s.handles(true,own,0));assert!(s.handles(true,Screen{container:false,..own},0));
    s.sprint_control=true;s.client_sprint=1;s.server_sprint=2;
    assert!(!s.sprint(true,own,false,0,true,false));assert!(s.sprint(true,own,false,1,true,false));
    assert!(!s.sprint(true,own,false,1,false,false));assert!(!s.sprint(true,own,false,2,true,true));
    s.sneak_control=true;s.client_sneak=1;assert!(s.sneak(true,own,false,false));
    let mut m=State::default();s.behavior=1;s.blink=true;s.maximum_time=100;m.configure(true,s,true);
    assert!(m.observe(own,1,1000));assert!(m.keyboard(0,true));assert!(m.pressed[0]);
    assert_eq!(m.blink(true),1);assert_eq!(m.blink(false),2);assert!(!m.expired(1099));assert!(m.expired(1100));
    assert!(!m.observe(own,1,1200));assert!(m.observe(own,2,1200));assert!(!m.expired(1299));
    assert!(!m.keyboard(0,false));assert!(!m.pressed[0]);
    m.observe(Screen::default(),2,1300);assert_eq!(m.blink(false),0);assert!(m.observe(own,2,1400));
}
#[test]
fn queue_and_native_policy() {
    use horizon_protocol::packets::play::Serverbound as S;
    use inventory_move as b;
    b::clear();b::configure(true,[3,0,1,1,2,1,2,0,1f32.to_bits() as i32,1,10000],false);
    b::observe(inventory_move_model::Screen{open:true,container:true,inventory:true,editing:false},1,0);
    assert_eq!(b::blink(true),1);assert_eq!(b::blink(false),2);assert_eq!(b::movement_sprint(true,-1),1);
    b::final_input([true,false,false,false,false,false,true]);
    let packets=[S::ContainerClick(()),S::ContainerButtonClick(()),S::SetCreativeModeSlot(()),S::ContainerSlotStateChanged(()),S::ContainerClose(())];
    for p in &packets{assert!(b::container_packet(p));assert!(b::delay(7,p));}
    assert!(!b::delay(7,&S::Other));assert!(b::next_loop().is_empty());
    let out=b::final_input([true;7]);assert_eq!(out,[false,false,false,false,false,false,true]);
    let batch=b::next_loop();assert_eq!(batch.len(),5);assert!(batch.iter().all(|(s,_)|*s==7));assert!(b::next_loop().is_empty());
    assert!(b::delay(7,&packets[0]));b::configure(false,[0;11],false);b::final_input([false;7]);assert!(b::next_loop().is_empty());
    b::configure(true,[1,0,0,0,0,0,0,0,0,0,0],true);assert!(b::keyboard(0,true));assert!(b::safe_click(0));assert!(!b::safe_click(1));
    b::SEARCH_FOCUSED.store(true,std::sync::atomic::Ordering::Release);
    b::observe(inventory_move_model::Screen{open:true,container:true,inventory:true,editing:false},2,0);
    assert!(!b::handles(0));assert!(!b::safe_click(0));b::SEARCH_FOCUSED.store(false,std::sync::atomic::Ordering::Release);b::clear();
}
