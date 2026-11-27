package net.ccbluex.liquidbounce.features.module.modules.render.debug
import fixture.Fixture
import net.minecraft.world.phys.Vec3
import net.ccbluex.liquidbounce.render.engine.type.Color4b
class DebuggedPoint(val center:Vec3,val color:Color4b,val radius:Double)
fun debugGeometry(name:String,body:()->DebuggedPoint){Fixture.trace.add("debugGeometry:$name");body()}
fun debugParameter(name:String,body:()->Any){Fixture.trace.add("debug:$name");body()}
