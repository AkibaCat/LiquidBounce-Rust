package net.ccbluex.liquidbounce.utils.entity
import fixture.Fixture
import fixture.Player
import net.minecraft.world.phys.Vec3
import net.ccbluex.liquidbounce.utils.movement.DirectionalInput
val Player.horizontalSpeed get()=speed
fun Player.isCloseToEdge(input:DirectionalInput,distance:Double):Boolean{Fixture.trace.add("edge:"+java.lang.Long.toUnsignedString(java.lang.Double.doubleToRawLongBits(distance)));return Fixture.edge}
fun Player.wouldBeCloseToFallOff(pos:Vec3):Boolean{Fixture.trace.add("fall");return Fixture.fall}
object PlayerSimulationCache{fun getSimulationForLocalPlayer()=this;fun getSnapshotAt(tick:Int):Snapshot{Fixture.trace.add("next");return Snapshot(Fixture.next)}}
data class Snapshot(val pos:Vec3)
