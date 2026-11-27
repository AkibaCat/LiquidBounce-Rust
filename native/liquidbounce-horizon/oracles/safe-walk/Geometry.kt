package net.ccbluex.liquidbounce.utils.math
import net.minecraft.world.phys.Vec3
import kotlin.math.atan2
import net.minecraft.util.Mth
data class BlockPos(val x:Int,val y:Int,val z:Int)
val BlockPos.bottomCenter get()=Vec3(x+0.5,y.toDouble(),z+0.5)
val Vec3.yaw get()=atan2(-x,z).toFloat()*Mth.RAD_TO_DEG
