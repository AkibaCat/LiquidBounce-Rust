// fixture code
private var externalFloors=false
private var follower=Follower(ctx,actions)
private fun followerSnapshot()=listOf(field(follower,"index"),follower.isFinished,follower.isMoving,follower.building,follower.target?.raw()?:"-",follower.sprint,follower.dive,follower.isPlainWalk,follower.safeWalk,follower.view?.raw()?:"-",follower.stuckAt?.raw()?:"-",follower.bridgeYRot?.raw()?:"-",follower.bridgeTarget?.raw()?:"-",follower.remainingBreaks.joinToString(";"){it.raw()},follower.remainingPlaces.joinToString(";"){it.raw()},follower.overshoots,follower.ahead.joinToString(";"){it.raw()}).joinToString("~")
private fun followerExtra(a:List<String>):String {
 val result=when(a[0]){
 "fnew"->{externalFloors=false;follower=Follower(ctx,actions);"-"}
 "ffollow"->{follower.follow(route!!,a[1].toInt());"-"}
 "fclear"->{follower.clear();"-"}
 "fcorrect"->{follower.positionCorrected();"-"}
 "fexternal"->{externalFloors=a[1].toBoolean();"-"}
 "fwalk"->{val keys=maskKeys(a[2]);"${follower.tick(keys,f(a[1])).name}:${keys.toInput(false).hashCode()}"}
 "foff"->follower.isOffPath().toString()
 "fnext"->follower.nextBlock?.raw()?:"-"
 "fruns"->"${follower.bridgeAhead}:${follower.swimAhead.joinToString(";"){it.to.raw()}}"
 "fpoint"->follower.pointAhead(v(a[1]),d(a[2]))?.raw()?:"-"
 "flook"->follower.lookAhead(v(a[1]),v(a[2]),d(a[3]))?.raw()?:"-"
 "fturn"->follower.turnAhead(d(a[1])).raw()
 "fhead"->follower.head(f(a[1])).raw()
 else->error(a.joinToString(" "))
 }
 return result+"@"+followerSnapshot()+"@"+actionSnapshot()
}
