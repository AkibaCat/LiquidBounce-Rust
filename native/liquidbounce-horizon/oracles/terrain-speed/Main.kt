import com.google.gson.*
import fixture.FixtureState as F
import net.ccbluex.liquidbounce.features.module.modules.movement.terrainspeed.ModuleTerrainSpeed as M
import net.ccbluex.liquidbounce.features.module.FixtureInput
import net.ccbluex.liquidbounce.config.types.group.*
import net.ccbluex.liquidbounce.event.Hooks
import net.ccbluex.liquidbounce.event.events.*
import net.minecraft.core.*
import net.minecraft.world.entity.MoverType
import net.minecraft.world.entity.player.Input
import net.minecraft.world.phys.Vec3
import net.minecraft.world.level.block.Blocks

fun group(root:ValueGroup,name:String)=root.children.filterIsInstance<ValueGroup>().first{it.name==name}
@Suppress("UNCHECKED_CAST")
fun bool(root:ValueGroup,name:String,value:Boolean){(root.children.filterIsInstance<Value<*>>().first{it.name==name} as Value<Boolean>).value=value}
@Suppress("UNCHECKED_CAST")
fun float(root:ValueGroup,name:String,value:Float){(root.children.filterIsInstance<Value<*>>().first{it.name==name} as Value<Float>).value=value}
fun vector(row:JsonObject,name:String):Vec3 {val v=row.getAsJsonArray(name);return Vec3(v[0].asDouble,v[1].asDouble,v[2].asDouble)}
fun bits(v:Vec3)=listOf(v.x.toRawBits().toULong().toString(),v.y.toRawBits().toULong().toString(),v.z.toRawBits().toULong().toString())
fun configure(row:JsonObject) {
 F.running=row["running"].asBoolean
 F.player=F.newPlayer();F.player.input=FixtureInput()
 F.player.fixtureVelocity=vector(row,"velocity");F.player.fixturePosition=vector(row,"position")
 F.player.xo=F.player.fixturePosition.x;F.player.yo=F.player.fixturePosition.y;F.player.zo=F.player.fixturePosition.z
 F.player.startBlockY=BlockPos.containing(F.player.fixturePosition).y
 F.player.horizontalCollision=row["collision"].asBoolean;F.player.climbing=row["climbable"].asBoolean;F.player.water=row["water"].asBoolean
 F.player.fixtureDirection=Direction.values()[row["direction"].asInt]
 F.player.yRot=row["yaw"].asFloat
 val input=F.player.input as FixtureInput
 val flags=row["input"].asInt;input.original=Input(flags and 1!=0,flags and 2!=0,flags and 4!=0,flags and 8!=0,false,false,false)
 input.vector(if(row["moving"].asBoolean)1f else 0f,0f)
 F.client.options.keyUp.down=row["forward"].asBoolean;F.client.options.keySprint.down=row["sprintKey"].asBoolean
 F.client.options.keyJump.down=row["jumpKey"].asBoolean;F.client.options.keyShift.down=row["sneakKey"].asBoolean
 F.pattern=row["pattern"].asInt
 val climb=group(M,"FastClimb");bool(climb,"Enabled",row["climbEnabled"].asBoolean)
 @Suppress("UNCHECKED_CAST")val modes=group(climb,"Mode") as ModeValueGroup<Mode>
 modes.activeMode=modes.modes.first{it.name==row["climbMode"].asString};float(modes.modes.first{it.name=="Motion"},"Motion",row["climbMotion"].asFloat)
 val ice=group(M,"IceSpeed");bool(ice,"Enabled",row["iceEnabled"].asBoolean);float(ice,"Slipperiness",row["slipperiness"].asFloat)
 val iceMotion=group(ice,"Motion");bool(iceMotion,"Enabled",row["iceMotionEnabled"].asBoolean);float(iceMotion,"Motion",row["iceMotion"].asFloat)
 val water=group(M,"WaterSpeed");bool(water,"Enabled",row["waterEnabled"].asBoolean);bool(water,"AutoSwim",row["autoSwim"].asBoolean)
 val base=group(water,"BaseSpeed");float(base,"Horizontal",row["baseHorizontal"].asFloat);float(base,"Vertical",row["baseVertical"].asFloat)
 val sprint=group(water,"SprintSpeed");bool(sprint,"Enabled",row["sprintEnabled"].asBoolean);float(sprint,"Horizontal",row["sprintHorizontal"].asFloat);float(sprint,"Vertical",row["sprintVertical"].asFloat)
 F.trace.clear();F.snaps.clear()
}
fun main(args:Array<String>){
 net.minecraft.SharedConstants.tryDetectVersion();net.minecraft.server.Bootstrap.bootStrap()
 // Force initialization and handler registration before any case.
 M.children.size
 val gson=GsonBuilder().setPrettyPrinting().create()
 val mcFacts=JsonObject()
 mcFacts.add("climbBlocks",gson.toJsonTree(net.minecraft.core.registries.BuiltInRegistries.BLOCK.filter{it is net.minecraft.world.level.block.LadderBlock || it is net.minecraft.world.level.block.VineBlock}.map{net.minecraft.core.registries.BuiltInRegistries.BLOCK.getKey(it).toString()}))
 val snapFacts=JsonArray()
 val limits=listOf(Double.NEGATIVE_INFINITY,-30000001.0,-30000000.0,-0.0,0.0,30000000.0,30000001.0,Double.POSITIVE_INFINITY,Double.NaN)
 for(x in limits)for(z in limits){val p=F.newPlayer();p.absSnapTo(x,0.125,z);val fact=JsonObject();fact.add("input",gson.toJsonTree(bits(Vec3(x,0.125,z))));fact.add("position",gson.toJsonTree(bits(p.fixturePosition)));fact.add("oldPosition",gson.toJsonTree(bits(Vec3(p.xo,p.yo,p.zo))));snapFacts.add(fact)}
 mcFacts.add("snaps",snapFacts)
 val directionFacts=JsonArray()
 for(yaw in listOf(Float.NEGATIVE_INFINITY,-3e15f,-181f,-135f,-90f,-45f,-0.0f,0.0f,45f,90f,135f,180f,3e15f,Float.POSITIVE_INFINITY,Float.NaN)){
  val p=F.newPlayer();p.yRot=yaw;val fact=JsonObject();fact.addProperty("yawBits",yaw.toRawBits().toUInt().toString());fact.addProperty("direction",Direction.fromYRot(yaw.toDouble()).name);directionFacts.add(fact)
 }
 mcFacts.add("directions",directionFacts)
 java.io.File(args[2]).writeText(gson.toJson(mcFacts))
 val source=JsonParser.parseString(java.io.File(args[0]).readText()).asJsonArray
 val output=JsonArray()
 for(entry in source){
  val row=entry.asJsonObject;configure(row)
  val result=JsonObject();result.add("input",row)
  when(row["event"].asString){
   "move"->{val e=PlayerMoveEvent(MoverType.values()[row["mover"].asInt],vector(row,"movement"));Hooks.emit(e);result.add("movementBits",gson.toJsonTree(bits(e.movement)))}
   "input"->{val e=MovementInputEvent(row["jump"].asBoolean);Hooks.emit(e);result.addProperty("jump",e.jump)}
   "tick"->Hooks.emit(PlayerTickEvent())
   "friction"->{val blocks=listOf(Blocks.ICE,Blocks.BLUE_ICE,Blocks.FROSTED_ICE,Blocks.PACKED_ICE,Blocks.SLIME_BLOCK,Blocks.AIR);val e=BlockSlipperinessMultiplierEvent(blocks[row["block"].asInt],row["friction"].asFloat);repeat(row["repeat"].asInt){Hooks.emit(e)};result.addProperty("frictionBits",e.slipperiness.toRawBits().toUInt().toString())}
  }
  result.add("positionBits",gson.toJsonTree(bits(F.player.fixturePosition)));result.add("velocityBits",gson.toJsonTree(bits(F.player.fixtureVelocity)))
  result.add("oldPositionBits",gson.toJsonTree(bits(Vec3(F.player.xo,F.player.yo,F.player.zo))))
  result.add("snaps",gson.toJsonTree(F.snaps.map(::bits)));result.add("trace",gson.toJsonTree(F.trace));output.add(result)
 }
 java.io.File(args[1]).writeText(gson.toJson(output))
}
