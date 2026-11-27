@file:OptIn(net.ccbluex.agent.InternalAgentApi::class)
package fixture
import java.io.File
import net.ccbluex.agent.pathfinder.route.*

private fun d(raw:String)=Double.fromBits(raw.toULong().toLong())
private class Graph(val start:Int,val goal:Int,val maximum:Double,val h:List<Double>,val edges:List<List<WeightedEdge<Int>>>,val beginMode:Int,val routeMode:Int,val partial:Boolean,limits:Limits):AStarSearch<Int,List<Int>>(limits){
 val contexts=ArrayList<String>()
 override fun begin():Outcome<List<Int>> = when(beginMode){1->Outcome.AlreadyThere;2->Outcome.Failed(Failure.INVALID_DESTINATION);3->Outcome.Searching;else->searchFrom(start,maximum)}
 override fun isGoal(node:Int)=node==goal
 override fun neighbors(node:Int):List<WeightedEdge<Int>>{
  val s=search!!
  contexts+="$node:"+edges.indices.joinToString(";"){"$it,${s.cost(it).toRawBits().toULong()},${s.previous(it)?:"-"}"}
  return edges[node]
 }
 override fun heuristic(node:Int)=h[node]
 override fun route(nodes:List<Int>):Outcome<List<Int>> = when(routeMode){1->Outcome.Searching;2->Outcome.Failed(Failure.TOO_FAR);else->Outcome.Found(nodes)}
 override fun closestOr(failure:Failure):Outcome<List<Int>> = if(partial)closestNodes{h[it]}?.let{Outcome.Found(it)}?:Outcome.Failed(failure)else Outcome.Failed(failure)
}
private fun outcome(result:Outcome<List<Int>>):String=when(result){Outcome.Searching->"S";Outcome.AlreadyThere->"A";is Outcome.Failed->"E:${result.failure}";is Outcome.Found->"F:"+result.value.joinToString(",")}
fun main(){
 val nanos=AStarSearch::class.java.getDeclaredField("computeNanos").also{it.isAccessible=true}
 var graph:Graph?=null
 val output=File(System.getProperty("commands")).readLines().map{command->
  val p=command.split(' ');var returned="-"
  when(p[0]){
   "graph"->{val count=p[1].toInt();val edges=MutableList(count){mutableListOf<WeightedEdge<Int>>()};if(p[6]!="-")for(raw in p[6].split(';')){val a=raw.split(',');edges[a[0].toInt()]+=WeightedEdge(a[1].toInt(),d(a[2]))};graph=Graph(p[2].toInt(),p[3].toInt(),d(p[4]),p[5].split(',').map(::d),edges,p[10].toInt(),p[11].toInt(),p[12].toBoolean(),Limits(100.0,maxVisitedNodes=p[7].toInt(),visitedNodesPerSlice=p[8].toInt(),maxComputeNanos=p[9].toLong()))}
   "seed"->nanos.setLong(graph!!,p[1].toLong())
   "advance"->try{returned=outcome(graph!!.advance())}catch(e:IllegalArgumentException){returned="invalid"}
   else->error(command)
  }
  val g=graph!!;val search=g.search
  val context=g.contexts.joinToString("/");g.contexts.clear()
  returned+"|"+nanos.getLong(g)+"|"+(search?.iterations?:"-")+"|"+(search?.isFinished?:"-")+"|"+(search?.closestPath()?.nodes?.joinToString(",")?:"-")+"|"+context
 }
 File(System.getProperty("results")).writeText(output.joinToString("\n")+"\n")
}
