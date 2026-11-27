import net.ccbluex.agent.world.item.*
import java.util.UUID
// fixture code
private class TargetStack(val id:Int,var tool:Boolean=false,var correct:Boolean=false,var speed:Float=1f,var efficiency:Int=0,var placeable:Boolean=false,var unfavourable:Boolean=false):ItemStack(){
 override val item get()=Item.AIR
 var captureCount=1;var captureEmpty=true
 override val count get():Int{events+="count:$id";return captureCount}
 override val isEmpty get():Boolean{events+="empty:$id";return captureEmpty}
 override val maxStackSize=64;override val damageValue=0;override val maxDamage=0;override val isDamageableItem=false
 override fun copy():ItemStack{events+="copy:$id";return TargetStack(id,tool,correct,speed,efficiency,placeable,unfavourable).also{it.captureCount=captureCount;it.captureEmpty=captureEmpty}}
 override fun getDestroySpeed(state:BlockState):Float{events+="speed:$id";return speed}
 override fun isCorrectToolForDrops(state:BlockState):Boolean{events+="harvest:$id";return correct}
 override val isTool get():Boolean{events+="tool:$id";return tool}
 override fun isToolCorrectForDrops(state:BlockState):Boolean{events+="correct:$id";return correct}
 override fun getEnchantmentLevel(enchantment:Enchantment):Int{events+="eff:$id";return efficiency}
 override val enchantments=emptyMap<Enchantment,Int>()
 override val food:FoodProperties?=null
 override val potionEffects:List<MobEffectInstance>?=null
 override fun computeAttribute(attribute:Attribute,base:Double,slot:EquipmentSlot)=base
 override val equipmentSlot:EquipmentSlot?=null
 override val useEffects=UseEffects.DEFAULT
 override val fireworkFlightDuration:Int?=null
 override fun canGlideUsing(slot:EquipmentSlot)=false
 override fun sameData(other:ItemStack)=this===other
}
private var targetStacks=(0..40).associateWith{TargetStack(it)}.toMutableMap()
private var selectedSlot=0
private val targetInventory=Proxy.newProxyInstance(Inventory::class.java.classLoader,arrayOf(Inventory::class.java)){_,method,args->when(method.name){
 "getItem"->{val slot=args!![0]as Int;events+="I:$slot";targetStacks.getValue(slot)}
 "getSelectedSlot"->{events+="slot";selectedSlot}
 else->error("Inventory.${method.name}")
}}as Inventory
private val targetPolicy=object:PlaceableBlocks{
 override fun isPlaceable(stack:ItemStack):Boolean{val s=stack as TargetStack;events+="P:${s.id}";return s.placeable}
 override fun isUnfavourable(stack:ItemStack):Boolean{val s=stack as TargetStack;events+="Q:${s.id}";return s.unfavourable}
}
private var targetReach=4.5
private var targetUuid=0L
private var targetViewStep=0f
private var targetHitPos=BlockPos.ZERO
private var targetHitFace=Direction.UP
private var targetHit=true
private var clipCount=0
private var clipLimit=Int.MAX_VALUE
private fun targetClip(context:ClipContext):BlockHitResult {
 events+="Z:${context.from.raw()}:${context.to.raw()}";clipCount++
 return if(targetHit&&clipCount<=clipLimit)BlockHitResult(context.to,targetHitFace,targetHitPos,false)else BlockHitResult.miss(context.to,targetHitFace,targetHitPos)
}
private fun BlockTargets.Plan.raw()=listOf(pos.raw(),against.raw(),face.name,point.raw(),rotation.raw(),slot,hand.name).joinToString("^")
private fun targetsExtra(a:List<String>):String=when(a[0]){
 "targetnew"->{targetStacks=(0..40).associateWith{TargetStack(it)}.toMutableMap();selectedSlot=0;targetReach=4.5;targetUuid=0;targetViewStep=0f;targetHitPos=BlockPos.ZERO;targetHitFace=Direction.UP;targetHit=true;clipCount=0;clipLimit=Int.MAX_VALUE;"-"}
 "targetstack"->{targetStacks[a[1].toInt()]=TargetStack(a[1].toInt(),a[2].toBoolean(),a[3].toBoolean(),f(a[4]),a[5].toInt(),a[6].toBoolean(),a[7].toBoolean());"-"}
 "targetsetup"->{selectedSlot=a[1].toInt();targetReach=d(a[2]);targetUuid=a[3].toLong();targetViewStep=f(a[4]);targetHitPos=p(a[5]);targetHitFace=Direction.valueOf(a[6]);targetHit=a[7].toBoolean();clipLimit=a[8].toInt();clipCount=0;"-"}
 "targetplan"->BlockTargets(ctx).planPlacement(p(a[1]),a[2].toInt(),a[3].toBoolean())?.raw()?:"-"
 "targetheld"->BlockTargets(ctx).heldClick(p(a[1]),ViewRotation(f(a[2]),f(a[3])))?.raw()?:"-"
 "targetaim"->BlockTargets(ctx).aimAt(p(a[1]))?.raw()?:"-"
 "targetclicks"->BlockTargets(ctx).clicksFrom(v(a[1]),ViewRotation(f(a[2]),f(a[3])),p(a[4])).toString()
 "targetdig"->BlockTargets(ctx).diggingSlot(states.getValue(a[1].toInt()))?.toString()?:"-"
 "targetslot"->BlockTargets(ctx).placeableSlot()?.toString()?:"-"
 "clickpoint"->clickPoint(p(a[1]),Direction.valueOf(a[2]),a[3].toLong()).raw()
 else->error(a.joinToString(" "))
}
