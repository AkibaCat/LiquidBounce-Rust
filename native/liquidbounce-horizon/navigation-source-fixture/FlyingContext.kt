package net.ccbluex.agent.navigation
import net.ccbluex.agent.math.Vec3
internal class NavigationContext(val body:Body,val route:FlightReport)
internal class FlightReport {
 fun following(points:List<Vec3>,progress:Progress){fixture.Probe.events+="R:F:"+points.joinToString(";",transform=fixture.Probe::vector)+":"+progress.isMoving+":"+progress.ahead.joinToString(";",transform=fixture.Probe::vector)}
 fun searching(start:Vec3,goal:Vec3,radius:Double,preview:List<Vec3>?){fixture.Probe.events+="R:S:${fixture.Probe.vector(start)}:${fixture.Probe.vector(goal)}:${fixture.Probe.raw(radius)}:"+(preview?.joinToString(";",transform=fixture.Probe::vector)?:"-")}
 fun invalid(){fixture.Probe.events+="R:I"}
 fun done(){fixture.Probe.events+="R:D"}
}
