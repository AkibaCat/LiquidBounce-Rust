import com.google.gson.*
import java.io.File
import net.ccbluex.liquidbounce.features.module.Fixture
import net.ccbluex.liquidbounce.features.module.modules.misc.ModuleGUICloser
import net.ccbluex.liquidbounce.event.events.ScreenEvent
import net.minecraft.network.chat.Component
import net.minecraft.client.gui.screens.Screen
import net.minecraft.client.gui.screens.inventory.AbstractContainerScreen
fun main(args:Array<String>) {
 val gson=GsonBuilder().serializeNulls().create()
 check(ModuleGUICloser.name=="GUICloser")
 val result=JsonArray()
 for(row in JsonParser.parseString(File(args[0]).readText()).asJsonArray) {
  val v=row.asJsonObject
  val out=JsonObject()
  try {
   val filters=v["patterns"].asJsonArray.map { if(it.isJsonNull)null else Regex(it.asString) }.toMutableList()
   @Suppress("UNCHECKED_CAST")
   val actual=filters as MutableList<Regex>
   Fixture.filters=actual;Fixture.mode=v["mode"].asString;Fixture.print=v["print"].asBoolean
   Fixture.running=v["running"].asBoolean;Fixture.messages.clear()
   val title=Component(v["title"].asString)
   val screen=when(v["screen"].asString){"container"->AbstractContainerScreen<Any>(title);"other"->Screen(title);else->null}
   val event=ScreenEvent(screen)
   var nullFilter=false
   try{Fixture.handler(event)}catch(e:NullPointerException){nullFilter=true}
   out.addProperty("valid",true);out.addProperty("cancel",event.cancelled)
   out.addProperty("print",Fixture.messages.isNotEmpty());out.addProperty("nullFilter",nullFilter)
   out.add("messages",gson.toJsonTree(Fixture.messages.map{line->line.map{mapOf("text" to it.string,"kind" to it.kind,"copied" to it.copied)}}))
  } catch(e:java.util.regex.PatternSyntaxException) {out.addProperty("valid",false)}
  result.add(out)
 }
 System.out.write(gson.toJson(result).toByteArray(Charsets.UTF_8))
}
