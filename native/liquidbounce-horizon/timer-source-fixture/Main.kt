package fixture
import java.io.File
import net.ccbluex.liquidbounce.utils.client.Timer
import net.ccbluex.liquidbounce.utils.kotlin.Priority
import net.ccbluex.liquidbounce.features.module.ClientModule
import net.ccbluex.liquidbounce.event.Events
import net.ccbluex.liquidbounce.event.events.*
import net.ccbluex.liquidbounce.event.CoroutineTicker
import net.ccbluex.liquidbounce.event.cancelInactive
import net.ccbluex.liquidbounce.features.module.modules.world.ModuleTimer as M
import net.ccbluex.liquidbounce.config.types.group.Value
import net.ccbluex.liquidbounce.utils.combat.CombatManager
class Player {var movingFlag=false}
object Gui {var current:Any?=null;fun screen()=current}
object Minecraft { var isSameThread=true;var player:Player?=Player();val gui=Gui;fun execute(action:()->Unit){action()} }
val providers=mutableMapOf<Long,ClientModule>()
fun provider(id:String)=providers.getOrPut(id.toLong()){ClientModule()}
fun f(bits:String)=Float.fromBits(bits.toLong().toInt())
fun privateInt(name:String)=Timer.javaClass.getDeclaredField(name).also{it.isAccessible=true}.getInt(Timer)
fun privateFloat(name:String)=Timer.javaClass.getDeclaredField(name).also{it.isAccessible=true}.getFloat(Timer)
fun state():String=listOf(Timer.balance.toRawBits().toUInt(),privateFloat("frameSpeed").toRawBits().toUInt(),
 Timer.realTicks.toRawBits().toULong(),privateInt("burstTicks"),Timer.bursting,privateInt("heldTicks"),privateInt("frameEnd"),Timer.timerSpeed.toRawBits().toUInt(),
 (0..21).joinToString(","){Timer.heldTicksWithin(it).toString()}).joinToString("|")
fun main() {
 Timer.timerSpeed
 M.running=false
 val result=File(System.getProperty("commands")).readLines().map { line ->
  val p=line.split(" ");var returned="-"
  try { when(p[0]) {
   "provider"->provider(p[1]).running=p[2].toBoolean()
   "same"->Minecraft.isSameThread=p[1].toBoolean()
   "player"->Minecraft.player=if(p[1].toBoolean())Player()else null
   "request"->Timer.requestTimerSpeed(f(p[4]),Priority(p[2].toInt()),provider(p[1]),p[3].toInt())
   "tick"->repeat(p[1].toInt()){CoroutineTicker.beginMinecraftTick();CoroutineTicker.tick();Events.emit(GameTickEvent());CoroutineTicker.endMinecraftTick()}
   "scale"->returned=Timer.scaleFrame(f(p[1])).toRawBits().toUInt().toString()
   "frame"->returned=Timer.frameTicks(p[1].toInt(),p[2].toInt()).toString()
   "playerTick"->Events.emit(PlayerTickEvent(p[1].toBoolean()))
   "hold"->Timer.hold(provider(p[1]),p[2].toInt())
   "burst"->returned=Timer.burst(p[1].toInt(),p[2].toInt()..p[3].toInt()).toString()
   "within"->returned=Timer.balanceWithin(p[1].toInt()..p[2].toInt()).toRawBits().toUInt().toString()
   "balanced"->returned=Timer.requestBalancedTimerSpeed(f(p[4]),Priority(p[2].toInt()),provider(p[1]),p[5].toInt()..p[6].toInt()).toString()
   "world"->Events.emit(WorldChangeEvent())
   "reset"->Timer.resetBalance()
   "module"->{val old=M.running;M.running=p[1].toBoolean();if(old && !M.running)M.onDisabled();cancelInactive();if(!old && M.running)M.modes.active.enable()}
   "mode"->{val target=M.modes.choices.first{it.name==p[1]};if(M.modes.active!==target){M.modes.active=target;cancelInactive();if(M.running)M.modes.active.enable()}}
   "context"->{Minecraft.player!!.movingFlag=p[1].toBoolean();CombatManager.isInCombat=p[2].toBoolean();Gui.current=when(p[3]){"inventory"->net.minecraft.client.gui.screens.inventory.InventoryScreen();"container"->net.minecraft.client.gui.screens.inventory.ContainerScreen();else->null}}
   "set"->{val mode=M.modes.choices.first{it.name==p[1]};val value=mode.values[p[2]]!!;@Suppress("UNCHECKED_CAST") when(value.value){is Float->(value as Value<Float>).value=f(p[3]);is Int->(value as Value<Int>).value=p[3].toInt();is Boolean->(value as Value<Boolean>).value=p[3].toBoolean();is IntRange->(value as Value<IntRange>).value=p[3].toInt()..p[4].toInt()}}
   "flag"->Events.emit(PacketEvent(net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket()))
   else->error(line)
  }}catch(e:IllegalArgumentException){returned="invalid-range"}
  returned+"|"+state()
 }
 File(System.getProperty("results")).writeText(result.joinToString("\n")+"\n")
}
