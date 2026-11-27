__FIGHT_GOAL_MODEL__
#[cfg(test)]
mod tests {
    use super::*;
    fn settings() -> Settings {
        Settings {dangerous_yaw:55.,runaway:true,leader:false,leader_radius:5.}
    }
    fn target() -> Opponent {
        Opponent {position:[0.;3],yaw:0.,range:2.,required_yaw:180.,out_of_distance:false,out_of_danger:false}
    }
    #[test]
    fn fight_goal_smoke_leader_preempts_combat_and_radius_equality_jumps() {
        let mut s=settings();s.leader=true;
        let goal=select([0.;3],Some(target()),Some([10.,0.,0.]),s,false,|yaw|if yaw == -90.{[-1.,0.,0.]}else{[1.,0.,0.]},|_|panic!("leader never collision filters"));
        assert_eq!(goal,Some(([5.,0.,0.],true)));
    }
    #[test]
    fn fight_goal_smoke_cooldown_runaway_starts_at_player() {
        let result=select([3.,2.,1.],Some(target()),None,settings(),false,|_|[-1.,0.,0.],|_|panic!("runaway never collision filters"));
        assert_eq!(result,Some(([1.,2.,1.],false)));
    }
    #[test]
    fn fight_goal_smoke_stable_look_sort_decides_equal_player_distance() {
        let mut s=settings();s.runaway=false;s.dangerous_yaw=0.;
        let mut t=target();t.range=1.;
        let mut calls=0;
        let result=select([0.;3],Some(t),None,s,true,|yaw|match yaw as i32 {-180=>[-1.,0.,0.],-170=>[1.,0.,0.],_=>[10.,0.,0.]},|at|{calls+=1;at[0]==10.});
        assert_eq!(result,Some(([1.,0.,0.],false)));
        assert_eq!(calls,37);
    }
    #[test]
    fn fight_goal_smoke_danger_angle_is_inclusive_and_plain_jump_matches_context() {
        let mut s=settings();s.runaway=false;s.dangerous_yaw=50.;
        let mut t=target();t.range=1.;t.out_of_distance=true;
        let mut calls=0;
        let result=select([55.,0.,0.],Some(t),None,s,true,|yaw|[f64::from(yaw),0.,0.],|_|{calls+=1;false});
        assert_eq!(result,Some(([60.,0.,0.],true)));
        assert_eq!(calls,37);
    }
    #[test]
    fn fight_goal_smoke_fallback_can_collide_and_missing_leader_still_assists() {
        let mut s=settings();s.runaway=false;s.leader=true;s.leader_radius=2.;
        let result=select([0.;3],Some(target()),None,s,true,|_|[2.,0.,0.],|_|true);
        assert_eq!(result,Some(([4.,0.,0.],true)));
        assert!(select([0.;3],None,None,s,true,|_|[0.;3],|_|false).is_none());
    }
    #[test]
    fn fight_goal_smoke_java_double_compare_and_first_ties() {
        assert_eq!(compare(f64::NAN,-f64::NAN),Ordering::Equal);
        assert_eq!(compare(f64::NAN,f64::INFINITY),Ordering::Greater);
        assert_eq!(compare(-0.,0.),Ordering::Less);
        assert_eq!(closest([[1.,0.,0.],[-1.,0.,0.]],[0.;3]),Some([1.,0.,0.]));
    }
}
}
