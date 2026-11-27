import com.google.gson.*
import java.io.File
import net.ccbluex.liquidbounce.features.module.Fixture
import net.ccbluex.liquidbounce.features.module.modules.misc.ModuleTextFieldProtect

fun main(args: Array<String>) {
    check(ModuleTextFieldProtect.name == "TextFieldProtect")
    val result = JsonArray()
    for (row in JsonParser.parseString(File(args[0]).readText()).asJsonArray) {
        val v = row.asJsonObject
        val out = JsonObject()
        try {
            Fixture.patterns = if (v["patterns"].isJsonNull) null else {
                val patterns = v["patterns"].asJsonArray.map { if (it.isJsonNull) null else Regex(it.asString) }.toMutableSet()
                @Suppress("UNCHECKED_CAST")
                (patterns as MutableSet<Regex>)
            }
            Fixture.running = v["running"].asBoolean
            out.addProperty("text", ModuleTextFieldProtect.protect(v["input"].asString, v["index"].asInt))
        } catch (t: Throwable) {
            out.addProperty("error", t.javaClass.simpleName)
        }
        result.add(out)
    }
    System.out.write(result.toString().toByteArray(Charsets.UTF_8))
}
