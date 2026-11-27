import com.google.gson.Gson
import com.google.gson.JsonObject
import fixture.Context
import fixture.Entry
import fixture.Player
import java.io.File
import java.util.Locale
import net.ccbluex.liquidbounce.features.module.modules.render.ModuleBetterInventory
import net.ccbluex.liquidbounce.config.types.group.Group
import net.ccbluex.liquidbounce.config.types.group.ModeValueGroup
import net.ccbluex.liquidbounce.config.types.group.Mode
import net.ccbluex.liquidbounce.render.engine.type.Color4b
import net.ccbluex.liquidbounce.utils.inventory.InventoryManager
import net.minecraft.client.gui.GuiGraphicsExtractor
import net.minecraft.world.inventory.Slot
import net.minecraft.world.item.ContainerComponent
import net.minecraft.world.item.ItemStack
fun main(args:Array<String>) {
    Locale.setDefault(Locale.US)
    val gson=Gson();val input=gson.fromJson(File(args[0]).readText(),Array<JsonObject>::class.java)
    val module=ModuleBetterInventory;val graphics=GuiGraphicsExtractor()
    val rows=input.map { row ->
        fun i(key:String)=row[key].asInt
        fun b(key:String)=row[key].asBoolean
        val state=row["state"].asJsonObject
        fun si(key:String)=state[key].asInt
        fun sb(key:String)=state[key].asBoolean
        fun sf(key:String)=Float.fromBits(si(key))
        val h=module.groups.getValue("HighlightClicked")
        h.set("Enabled",sb("highlight"))
        val modes=h.modeGroups.getValue("Mode")
        @Suppress("UNCHECKED_CAST")
        (modes as ModeValueGroup<Mode>).activeMode=modes.modes.first{it.name==state["highlightMode"].asString}
        modes.modes.first{it.name=="Border"}.set("Color",Color4b(si("highlightColor")))
        val t=module.groups.getValue("TextCooldownProgress")
        t.set("Enabled",sb("text"));t.set("Scale",sf("cooldownScale"));t.set("Color",Color4b(si("cooldownColor")))
        val choice=t.settings.getValue("Mode").value as Enum<*>
        t.set("Mode",choice.javaClass.enumConstants.first{(it as Enum<*>).name==state["cooldownMode"].asString})
        val c=module.groups.getValue("ContainerItemView")
        c.set("Enabled",sb("container"));c.set("SkipEmptyStack",sb("skipEmpty"));c.set("Scale",sf("containerScale"))
        c.set("RelativeToMouse",sb("relative"));c.set("RenderOffsetX",sf("offsetX"));c.set("RenderOffsetY",sf("offsetY"))
        module.set("MoveCursor",sb("move"));module.enabled=b("enabled");Context.inGame=b("inGame")
        Context.mc.player=if(b("player"))Player() else null
        Context.mc.player?.cooldowns?.let { it.progress=Float.fromBits(i("progress"));it.entry=if(row["entry"].isJsonNull)null else Entry(row["entry"].asJsonArray[0].asInt,row["entry"].asJsonArray[1].asInt) }
        InventoryManager.lastClickedSlot=i("last")
        val items=row["items"].asJsonArray.mapIndexed{index,value->ItemStack(index,!value.asBoolean)}
        val stack=ItemStack(-1,b("empty"),if(b("component"))ContainerComponent(items) else null)
        Context.ops.clear()
        with(module) {
            with(graphics) {
                drawHighlightSlot(Slot(i("slot"),i("x"),i("y")))
                try {drawTextCooldownProgress(stack,i("x"),i("y"))} catch(_:NullPointerException) {Context.ops+=mapOf("op" to "missingEntry")}
                var result=false
                if(b("carriedEmpty") && b("hover")) {
                    try {result=drawContainerItemView(stack,i("left"),i("top"),i("mouseX"),i("mouseY"))}
                    catch(_:IllegalArgumentException) {Context.ops+=mapOf("op" to "invalidScale")}
                }
                mapOf("move" to shouldMoveCursor,"preview" to result,"ops" to Context.ops.toList())
            }
        }
    }
    print(gson.toJson(mapOf("rows" to rows)))
}
