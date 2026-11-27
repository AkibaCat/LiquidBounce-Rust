package net.ccbluex.agent.pathfinder.glide
import fixture.GlideProbe
import net.ccbluex.agent.math.*
import net.ccbluex.agent.pathfinder.route.*
import net.ccbluex.agent.pathfinder.terrain.Terrain
import net.ccbluex.agent.pathfinder.level.BlockRegion
import net.ccbluex.agent.math.horizontalDistanceTo
import kotlin.math.*
public class RouteSearch(terrain:Terrain,private val start:Vec3,private val goal:Vec3,radius:Double,abilities:GlideAbilities):Search<Route>{
 private val program=GlideProbe.planProgram.toList();private val points=GlideProbe.points.toList();private val rockets=GlideProbe.routeRockets;private val approach=GlideProbe.approach;private var step=0
 init{GlideProbe.events+="F:G:${GlideProbe.vector(start)}:${GlideProbe.vector(goal)}:${GlideProbe.raw(radius)}:${abilities.rockets},${abilities.rocketTicks},${abilities.glideTicks}";terrain.state(0,0,0)}
 override fun advance():Outcome<Route>{val token=program[step.coerceAtMost(program.lastIndex)];GlideProbe.events+="A:G:$step";step++;return when(token[0]){'S'->Outcome.Searching;'F'->Outcome.Found(Route(start,points,goal,rockets,approach));'A'->Outcome.AlreadyThere;'E'->Outcome.Failed(Failure.valueOf(token.substring(1)));else->error("controlled glide search failure")}}
 override fun preview():List<Vec3>?{GlideProbe.events+="Q:G:$step";return if(program[(step-1).coerceAtLeast(0).coerceAtMost(program.lastIndex)]=="S1")listOf(Vec3(step.toDouble(),0.0,0.0))else null}
 companion object{fun region(start:Vec3,goal:Vec3):BlockRegion{val margin=min(start.horizontalDistanceTo(goal),48.0).toInt()+2;return BlockRegion(Mth.floor(min(start.x,goal.x))-margin,Mth.floor(min(start.y,goal.y))-2,Mth.floor(min(start.z,goal.z))-margin,Mth.floor(max(start.x,goal.x))+margin,Mth.floor(max(start.y,goal.y))+24,Mth.floor(max(start.z,goal.z))+margin)}}
}
public class AirwaySearch(terrain:Terrain,private val start:Vec3,goal:Destination,clearance:Double,limits:Limits):Search<net.ccbluex.agent.pathfinder.flight.Route>{
 companion object{const val BODY_SIZE=0.6}
 private val program=GlideProbe.airProgram.toList();private val points=GlideProbe.points.toList();private val reaches=GlideProbe.reaches;private var step=0
 init{val checked=listOf(BlockPos(0,0,0),BlockPos(4,0,0),BlockPos(100,0,0)).joinToString(";"){"${goal.isGoal(it)},${GlideProbe.raw(goal.distance(it))}"};GlideProbe.events+="F:A:${GlideProbe.vector(start)}:${goal is StageDestination}:${GlideProbe.raw(clearance)}:${GlideProbe.raw(limits.maxLength)}:${limits.maxVisitedNodes}:${limits.visitedNodesPerSlice}:${limits.maxComputeNanos}:$checked";terrain.state(0,0,0)}
 override fun advance():Outcome<net.ccbluex.agent.pathfinder.flight.Route>{val token=program[step.coerceAtMost(program.lastIndex)];GlideProbe.events+="A:A:$step";step++;return when(token[0]){'S'->Outcome.Searching;'F'->Outcome.Found(net.ccbluex.agent.pathfinder.flight.Route(start,points,reaches));'A'->Outcome.AlreadyThere;'E'->Outcome.Failed(Failure.valueOf(token.substring(1)));else->error("controlled airway search failure")}}
 override fun preview():List<Vec3>?{GlideProbe.events+="Q:A:$step";return if(program[(step-1).coerceAtLeast(0).coerceAtMost(program.lastIndex)]=="S1")listOf(Vec3(step.toDouble(),0.0,0.0))else null}
}
