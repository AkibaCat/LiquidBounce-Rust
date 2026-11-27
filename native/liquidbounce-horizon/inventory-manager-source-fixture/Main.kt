package fixture
import com.google.gson.*
import java.nio.file.*
import java.util.concurrent.ThreadLocalRandom
import kotlin.coroutines.*
import kotlinx.coroutines.*
import net.ccbluex.liquidbounce.event.*
import net.ccbluex.liquidbounce.event.events.*
import net.ccbluex.liquidbounce.utils.client.*
import net.ccbluex.liquidbounce.utils.inventory.*
import net.ccbluex.liquidbounce.utils.kotlin.Priority
import net.minecraft.client.gui.screens.inventory.*
import net.minecraft.network.protocol.game.*
import net.minecraft.world.inventory.*
import net.minecraft.world.item.ItemStack
import net.ccbluex.liquidbounce.features.global.GlobalSettingsInventory
import net.ccbluex.liquidbounce.features.module.modules.render.ModuleBetterInventory
import net.ccbluex.liquidbounce.features.module.modules.player.cheststealer.features.FeatureSilentScreen

class Owner:EventListener { override var running=true;var guard=true;val constraints=InventoryConstraints() }
object Harness {
    val effects=mutableListOf<String>()
    var menuProvider=true
    val owners=mutableMapOf<Int,Owner>()
    val plans=mutableMapOf<Int,InventoryPlan>()
    val jobs=mutableListOf<Job>()
    var nearest:List<String>?=null
    fun send(packet:Any):Boolean {
        val event=PacketEvent(packet);Events.fire(event)
        if(event.isCancelled)effects.add("cancel-packet")
        return !event.isCancelled
    }
    fun field(name:String):Any?=InventoryManager.javaClass.getDeclaredField(name).also { it.isAccessible=true }.get(InventoryManager)
    fun owner(id:Int)=owners.getOrPut(id) { Owner() }
    fun slot(value:JsonElement):ItemSlot {
        val tag=value.asString
        return if(tag.startsWith("p"))PlayerItemSlot(tag.drop(1).toInt())else ContainerItemSlot(tag.drop(1).toInt())
    }
    fun action(value:JsonObject):InventoryAction {
        val kind=value["kind"].asString
        val screen=mc.gui.screen() as? AbstractContainerScreen<*>
        return when(kind) {
            "click" -> InventoryAction.Click(if(value["screen"]?.asBoolean==true)screen else null,slot(value["slot"]),value["button"]?.asInt ?:1,ContainerInput.entries[value["input"]?.asInt ?:4])
            "creative" -> InventoryAction.Creative(ItemStack(value["item"]?.asString ?:"stone",value["count"]?.asInt ?:64),value["slot"]?.let {slot(it)})
            "close" -> InventoryAction.CloseScreen(screen!!)
            "use" -> InventoryAction.UseItem(HotbarItemSlot.entries.first { it.inventorySlot==value["slot"].asInt })
            else -> error(kind)
        }
    }
    fun command(command:JsonObject) {
        fun int(key:String,default:Int=0)=command[key]?.asInt ?:default
        fun bool(key:String,default:Boolean=false)=command[key]?.asBoolean ?:default
        when(command["cmd"].asString) {
            "tick" -> Events.tick()
            "advance" -> Clock.advance(Clock.now+int("ms"))
            "settings" -> {
                command["responseTime"]?.let { GlobalSettingsInventory.responseTime=it.asInt }
                command["lockAfterRejections"]?.let { GlobalSettingsInventory.lockAfterRejections=it.asInt }
            }
            "cursor" -> {
                val value=command["value"]
                val cursor=if(value==null || value.isJsonNull)null else org.joml.Vector2f(value.asJsonArray[0].asFloat,value.asJsonArray[1].asFloat)
                InventoryManager.javaClass.getDeclaredField("cursor").also { it.isAccessible=true }.set(InventoryManager,cursor)
            }
            "nearestFirst" -> {
                val slots=command["slots"].asJsonArray.map { it.asString }
                nearest=InventoryManager.nearestFirst(slots) { slot(JsonPrimitive(it)) }
            }
            "visiblecursor" -> {
                command["enabled"]?.let { ModuleBetterInventory.shouldMoveCursor=it.asBoolean }
                command["grabbed"]?.let { mc.mouseHandler.isMouseGrabbed=it.asBoolean }
                command["hidden"]?.let { FeatureSilentScreen.shouldHide=it.asBoolean }
                command["x"]?.let { mc.mouseHandler.x=it.asDouble }
                command["y"]?.let { mc.mouseHandler.y=it.asDouble }
                command["width"]?.let { mc.window.screenWidth=it.asInt }
                command["guiWidth"]?.let { mc.window.guiScaledWidth=it.asInt }
                (mc.gui.screen() as? AbstractContainerScreen<*>)?.let { screen ->
                    command["left"]?.let { screen.leftPos=it.asInt };command["top"]?.let { screen.topPos=it.asInt }
                }
            }
            "owner" -> { val owner=owner(int("id"));owner.running=bool("running",true);owner.guard=bool("guard",true) }
            "constraints" -> {
                val c=owner(int("owner",1)).constraints
                for(name in listOf("start","click","close","miss"))command[name]?.let {
                    val range=it.asJsonArray[0].asInt..it.asJsonArray[1].asInt
                    when(name){"start"->c.startDelay=range;"click"->c.clickDelay=range;"close"->c.closeDelay=range;else->c.missChance=range}
                }
                command["multiplier"]?.let { c.slotMultiplier=it.asFloat };command["requires"]?.let { c.mask=it.asInt }
            }
            "plan" -> {
                val id=int("id");val owner=owner(int("owner",1));val plan=InventoryPlan(owner,owner.constraints);plans[id]=plan
                val chains=command["chains"].asJsonArray.map { chain->chain.asJsonArray.map { action(it.asJsonObject) } }
                val priority=Priority.entries[int("priority",1)]
                jobs+=owner.eventListenerScope.launch(owner.wrapContinuationInterceptor(),start=CoroutineStart.UNDISPATCHED) {
                    try {
                        requires { owner.guard }
                        for((index,chain)in chains.withIndex()) { plan.perform(chain,priority);effects.add("completed:$id:$index") }
                        effects.add("done:$id")
                    } catch(e:CancellationException) { effects.add("cancel:$id") }
                }
                Clock.flush()
            }
            "set" -> {
                val slot=slot(command["slot"]);val server=slot.getIdForServer(if(command["screen"]?.asBoolean==true)mc.gui.screen() as? AbstractContainerScreen<*>else null)!!
                val menu=if(command["screen"]?.asBoolean==true)player.containerMenu else player.inventoryMenu
                menu.slots[server].item=ItemStack(command["item"]?.asString ?:"stone",int("count",64))
            }
            "carried" -> player.containerMenu.carried=ItemStack(command["item"]?.asString ?:"stone",int("count",64))
            "screen" -> {
                val kind=command["kind"].asString
                val id=int("id",7);val own=int("slots",27)
                val menu=if(kind=="container") {
                    val slots=MutableList(own) { MenuSlot(it%9*18,it/9*18) }
                    for(i in (9..35)+(36..44))slots.add(MenuSlot(slots.size%9*18,slots.size/9*18,container=player.inventory,source=player.inventoryMenu.slots[i]))
                    AbstractContainerMenu(id,slots)
                } else player.inventoryMenu
                player.containerMenu=menu
                mc.gui.value=when(kind) {"none"->null;"inventory"->InventoryScreen(menu);"container"->ContainerScreen(menu);"creative"->CreativeModeInventoryScreen(menu);else->net.minecraft.client.gui.screens.Screen()}
                Events.fire(ScreenEvent(mc.gui.screen()))
            }
            "state" -> {
                command["creative"]?.let { player.isCreative=it.asBoolean };command["game"]?.let { inGame=it.asBoolean }
                command["moving"]?.let { player.movement.set(if(it.asBoolean)1f else 0f,0f) }
                command["jumping"]?.let { player.jumping=it.asBoolean };command["rotation"]?.let { player.rotationMatches=it.asBoolean }
                command["using"]?.let { player.usingItem=it.asBoolean };command["breaking"]?.let { player.breaking=it.asBoolean }
                command["combat"]?.let { player.combat=it.asBoolean };command["serverControlled"]?.let { interaction.isServerControlledInventory=it.asBoolean }
            }
            "packet" -> when(command["kind"].asString) {
                "open-pending"->send(ServerboundInteractPacket())
                "opened"->send(ClientboundOpenScreenPacket())
                "out-close"->send(ServerboundContainerClosePacket(int("id")))
                "in-close"->send(ClientboundContainerClosePacket(int("id")))
                "slot"->{ val stack=ItemStack(command["item"]?.asString ?:"stone",int("count",64));send(ClientboundContainerSetSlotPacket(int("id"),int("slot"),stack));if(int("id")==player.containerMenu.containerId)player.containerMenu.slots.getOrNull(int("slot"))?.item=stack }
                "content"->send(ClientboundContainerSetContentPacket(int("id"),player.containerMenu.slots.map {it.item.copy()}))
                else->error(command.toString())
            }
            "world" -> Events.fire(WorldChangeEvent())
            else -> error(command.toString())
        }
        Clock.flush()
    }
    fun row(seed:Long):JsonObject {
        val row=JsonObject();row.addProperty("now",Clock.now);row.addProperty("seed",seed.toULong().toString())
        row.addProperty("open",InventoryManager.isInventoryOpenServerSide);row.addProperty("last",InventoryManager.lastClickedSlot)
        row.addProperty("locked",InventoryManager.serverLockedCount);row.addProperty("revert",InventoryManager.isRevertPending)
        row.addProperty("containerPending",InventoryManager.isContainerOpenPending);row.addProperty("usePending",InventoryManager.isUseItemPending)
        row.addProperty("sourceMillis",Clock.sourceMillis())
        row.add("nearest",Gson().toJsonTree(nearest))
        row.add("mouse",Gson().toJsonTree(listOf(mc.mouseHandler.x.toRawBits(),mc.mouseHandler.y.toRawBits())))
        row.addProperty("pending",(field("requests") as Collection<*>).size)
        val cursor=field("cursor") as? org.joml.Vector2f
        row.add("cursor",if(cursor==null)JsonNull.INSTANCE else JsonArray().also { it.add(cursor.x.toRawBits());it.add(cursor.y.toRawBits()) })
        row.add("effects",Gson().toJsonTree(effects));effects.clear()
        row.add("stacks",Gson().toJsonTree(player.containerMenu.slots.map { if(it.item.isEmpty)"" else "${it.item.components}:${it.item.count}" }))
        row.add("carried",JsonPrimitive(if(player.containerMenu.carried.isEmpty)""else "${player.containerMenu.carried.components}:${player.containerMenu.carried.count}"))
        return row
    }
}
fun main(args:Array<String>) {
    // EventListenerScope wraps Main's ContinuationInterceptor, so delay falls
    // back to DefaultDelay. Route that real coroutine fallback onto Main/Clock.
    System.setProperty("kotlinx.coroutines.main.delay","true")
    Clock.startSourceClock()
    InventoryManager // Register the actual unchanged manager handlers before use.
    ThreadLocalRandom.current();kotlin.random.Random.nextBoolean()
    val threadSeed=Thread::class.java.getDeclaredField("threadLocalRandomSeed").also { it.isAccessible=true }
    threadSeed.setLong(Thread.currentThread(),305419896L)
    val commands=JsonParser.parseString(Files.readString(Path.of(args[0]))).asJsonArray
    val rows=JsonArray()
    for(command in commands) { Harness.command(command.asJsonObject);rows.add(Harness.row(threadSeed.getLong(Thread.currentThread()))) }
    val result=JsonObject();result.addProperty("thread",Thread.currentThread().threadId());result.add("rows",rows)
    Files.writeString(Path.of(args[1]),GsonBuilder().setPrettyPrinting().create().toJson(result))
}
