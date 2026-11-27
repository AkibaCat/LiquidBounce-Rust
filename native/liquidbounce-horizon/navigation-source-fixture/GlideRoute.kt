import net.ccbluex.agent.pathfinder.glide.*
// fixture code
private var glideSearch:RouteSearch?=null
private fun reflect(target:Any,name:String)=target.javaClass.getDeclaredField(name).also{it.isAccessible=true}.get(target)
private fun glideState(s:RouteSearch):String {
 val attempt=reflect(s,"attempt");val rockets=reflect(attempt,"rockets")as Rockets
 return listOf((reflect(attempt,"approach")as Double).raw(),reflect(attempt,"tick"),(reflect(attempt,"position")as Vec3).raw(),(reflect(attempt,"motion")as Vec3).raw(),rockets.left,rockets.boostTicks,rockets.canFire).joinToString(":")
}
private fun glideOutcome(result:net.ccbluex.agent.pathfinder.route.Outcome<Route>):String=when(result){
 net.ccbluex.agent.pathfinder.route.Outcome.Searching->"S"
 net.ccbluex.agent.pathfinder.route.Outcome.AlreadyThere->"A"
 is net.ccbluex.agent.pathfinder.route.Outcome.Failed->"E:${result.failure}"
 is net.ccbluex.agent.pathfinder.route.Outcome.Found->{val r=result.value;"F:${r.start.raw()}:${r.points.joinToString(";"){it.raw()}}:${r.landing.raw()}:${r.rockets}:${r.approach.raw()}"}
}
private fun extra(a:List<String>,world:World,terrain:Terrain):String=when(a[0]){
 "region"->RouteSearch.region(v(a[1]),v(a[2])).let{"${it.minX},${it.minY},${it.minZ},${it.maxX},${it.maxY},${it.maxZ}"}
 "glide"->{glideSearch=RouteSearch(Terrain(world,terrain.width,terrain.height,terrain.canOpenDoors),v(a[1]),v(a[2]),d(a[3]),GlideAbilities(a[4].toInt(),a[5].toInt(),a[6].toInt()));glideState(glideSearch!!)}
 "advance"->glideSearch!!.let{glideOutcome(it.advance())+"#"+glideState(it)}
 "preview"->glideSearch!!.preview().joinToString(";"){it.raw()}
 else->error(a)
}
