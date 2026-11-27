package net.ccbluex.agent.pathfinder.flight
import fixture.Probe
import net.ccbluex.agent.math.*
import net.ccbluex.agent.pathfinder.route.*
import net.ccbluex.agent.pathfinder.terrain.Terrain
public class RouteSearch(private val terrain:Terrain,private val start:Vec3,goal:Destination,abilities:FlyAbilities,limits:Limits):Search<Route> {
 private val program=Probe.program.toList()
 private val points=Probe.points.toList()
 private val reachable=Probe.reachable
 private var step=0
 init {
  val checked=listOf(BlockPos(0,0,0),BlockPos(4,0,0),BlockPos(100,0,0)).joinToString(";"){"${goal.isGoal(it)},${Probe.raw(goal.distance(it))}"}
  Probe.events+="F:${Probe.vector(start)}:${goal is StageDestination}:${abilities.control}:${Probe.raw(abilities.horizontalSpeed)}:${Probe.raw(abilities.verticalSpeed)}:${abilities.isFlying}:${Probe.raw(limits.maxLength)}:${limits.maxVisitedNodes}:$checked"
  terrain.state(0,0,0)
 }
 override fun advance():Outcome<Route> {
  val token=program[step.coerceAtMost(program.lastIndex)];Probe.events+="A:$step";step++
  return when(token[0]){'S'->Outcome.Searching;'F'->Outcome.Found(Route(start,points,reachable));'A'->Outcome.AlreadyThere;'E'->Outcome.Failed(Failure.valueOf(token.substring(1)));else->error("fixture search exception")}
 }
 override fun preview():List<Vec3>? {
  Probe.events+="Q:$step"
  return if(program[(step-1).coerceAtLeast(0).coerceAtMost(program.lastIndex)]=="S1")listOf(Vec3(step.toDouble(),0.0,0.0))else null
 }
}
