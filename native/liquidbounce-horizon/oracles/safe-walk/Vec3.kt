package net.minecraft.world.phys
data class Vec3(val x:Double,val y:Double,val z:Double){
 fun subtract(other:Vec3)=Vec3(x-other.x,y-other.y,z-other.z)
 fun horizontalDistanceSqr()=x*x+z*z
}
