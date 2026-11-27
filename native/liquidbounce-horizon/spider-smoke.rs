#[path="../../native/liquidbounce-behaviors/src/spider.rs"]mod spider;
use spider::{Mode,Spider,Velocity};
#[test]fn three_modes_preserve_motion_two_distinct_waits_and_polar_bounds(){
 let mut s=Spider::default();assert_eq!(s.game_tick(true,true,false).velocity,Some(Velocity::Vertical(f64::from(0.3f32))));assert!(s.game_tick(false,true,false).velocity.is_none());
 s.set_mode(Mode::Polar);let mut y=0.42;s.jump(true,true,&mut y);assert_eq!(y,0.55);assert!(spider::geometry::should_shrink(2,1.5,false,false));assert!(!spider::geometry::should_shrink(1,1.5,false,false));assert!(spider::geometry::should_shrink(-5,1.5,true,true));
 assert_eq!(spider::geometry::shrink_box([0.0,0.0,0.0,1.0,1.0,1.0]),Some([0.0001,0.0,0.0001,0.9999,1.0,0.9999]));assert!(spider::geometry::shrink_box([0.0,0.0,0.0,0.0002,1.0,1.0]).is_none());
 s.set_mode(Mode::Vulcan);let a=s.game_tick(true,true,false).wait.unwrap();assert!(s.game_tick(true,true,false).wait.is_none());assert!(s.callback(a,true).0);let(keep,step)=s.callback(a,true);assert!(!keep);assert_eq!(step.velocity,Some(Velocity::Vertical(9.6599696)));let b=step.wait.unwrap();assert_ne!(a,b);assert!(s.callback(b,true).0);assert_eq!(s.callback(b,true).1.velocity,Some(Velocity::All([0.0,0.0001,0.0])));
 assert_eq!(s.game_tick(true,false,false).velocity,Some(Velocity::Vertical(0.0)));assert!(!s.requires_stop);let c=s.game_tick(true,true,false).wait.unwrap();s.cancel();assert!(!s.callback(c,true).0);assert!(s.requires_stop);assert_eq!(Mode::parse("Vulcan288"),Some(Mode::Vulcan));
}
