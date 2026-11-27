package net.minecraft.client.player
import fixture.Fixture
class Profile(val name:String)
open class AbstractClientPlayer(val id:Int,name:String,var distanceSquared:Double=0.0,var isRemoved:Boolean=false){
 val gameProfile=Profile(name)
 val isDeadOrDying get()=Fixture.dead
 fun closerThan(player:AbstractClientPlayer,range:Double):Boolean {Fixture.trace.add("range:"+java.lang.Double.doubleToRawLongBits(range).toULong().toString());return distanceSquared<range*range}
}
