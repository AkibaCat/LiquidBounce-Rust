package net.ccbluex.agent.navigation
import net.ccbluex.agent.math.*
internal data class RuntimeOptions(val canElytra:Boolean=true,val elytraRockets:Boolean=true)
internal class NavigationContext(val body:Body,val route:RuntimeReport=RuntimeReport(),val listener:RuntimeListener=RuntimeListener()) {
 var current=RuntimeOptions()
 fun options()=current
}
internal class RuntimeReport {
 fun blocks(a:List<BlockPos>,b:List<BlockPos>){fixture.RuntimeProbe.events+="R:B:${a.size},${b.size}"}
 fun following(points:List<Vec3>,progress:Progress){fixture.RuntimeProbe.events+="R:F:"+points.joinToString(";",transform=fixture.RuntimeProbe::vector)+":"+progress.isMoving+":"+progress.ahead.joinToString(";",transform=fixture.RuntimeProbe::vector)}
}
internal class RuntimeListener {fun touched(pos:BlockPos,direction:Direction){fixture.RuntimeProbe.events+="T:${pos.x},${pos.y},${pos.z}:$direction"}}
