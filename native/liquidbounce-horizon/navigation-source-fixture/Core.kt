package fixture
import java.io.File
import net.ccbluex.agent.navigation.Keystrokes
import net.ccbluex.agent.world.Input
import net.ccbluex.agent.pathfinder.route.AStar
import net.ccbluex.agent.pathfinder.route.ShortestPath
import net.ccbluex.agent.pathfinder.route.WeightedEdge

fun number(bits:String)=Double.fromBits(bits.toULong().toLong())
private fun path(value:ShortestPath<Int>?):String=value?.let{it.nodes.joinToString(",")+":"+it.totalCost.toRawBits().toULong()}?:"-"
fun input(mask:Int)=Input(mask and 64!=0,mask and 32!=0,mask and 16!=0,mask and 8!=0,mask and 4!=0,mask and 2!=0,mask and 1!=0)
fun main(){
 val keys=Keystrokes();var count=0;var goal=0;var heuristics=doubleArrayOf();var graph=listOf<List<WeightedEdge<Int>>>();var search:AStar<Int>?=null
 val results=File(System.getProperty("commands")).readLines().map{command->
  val p=command.split(" ");var returned="-"
  when(p[0]){
   "key"->returned=keys.press(input(p[1].toInt()),p[2].toBoolean(),p[3].toInt(),p[4].toInt()).hashCode().toString()
   "resetKeys"->keys.reset()
   "graph"->{count=p[1].toInt();goal=p[3].toInt();heuristics=p[5].split(',').map(::number).toDoubleArray();val edges=MutableList(count){mutableListOf<WeightedEdge<Int>>()};if(p[6]!="-")for(edge in p[6].split(';')){val e=edge.split(',');edges[e[0].toInt()]+=WeightedEdge(e[1].toInt(),number(e[2]))};graph=edges;search=AStar(p[2].toInt(),{it==goal},{graph[it]},{heuristics[it]},number(p[4]))}
   "advance"->try{returned=path(search!!.advance(p[1].toInt()))}catch(e:IllegalArgumentException){returned="invalid"}
   else->error(command)
  }
  if(p[0]=="key"||p[0]=="resetKeys")returned else {
   val s=search!!
   returned+"|"+s.iterations+"|"+s.isFinished+"|"+path(s.closestPath())+"|"+(0 until count).joinToString(","){s.cost(it).toRawBits().toULong().toString()}+"|"+(0 until count).joinToString(","){s.previous(it)?.toString()?:"-"}
  }
 }
 File(System.getProperty("results")).writeText(results.joinToString("\n")+"\n")
}
