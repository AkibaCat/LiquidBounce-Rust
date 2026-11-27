import com.google.gson.*
import net.ccbluex.liquidbounce.config.gson.adapter.SimpleStringTypeAdapter
import net.ccbluex.liquidbounce.config.gson.stategies.*
import net.ccbluex.liquidbounce.config.gson.serializer.ValueGroupSerializer
import net.ccbluex.liquidbounce.config.types.*
import net.ccbluex.liquidbounce.config.types.group.ValueGroup
import net.ccbluex.liquidbounce.config.types.list.MutableListValue
import java.io.File
fun gson(serializer:ValueGroupSerializer,interop:Boolean):Gson = GsonBuilder()
    .addSerializationExclusionStrategy(if(interop) ProtocolExcludeStrategy else ExcludeStrategy)
    .registerTypeAdapter(Regex::class.java,SimpleStringTypeAdapter.KT_REGEX)
    .registerTypeHierarchyAdapter(ValueGroup::class.java,serializer).create()
fun main(args:Array<String>) {
    val file=gson(ValueGroupSerializer.FILE_SERIALIZER,false)
    val interop=gson(ValueGroupSerializer.INTEROP_SERIALIZER,true)
    val public=gson(ValueGroupSerializer.PUBLIC_SERIALIZER,false)
    val out=JsonArray()
    for(input in JsonParser.parseString(File(args[0]).readText()).asJsonArray) {
        val filter=MutableListValue("Filter",mutableListOf(Regex("^Vote$")),ValueType.TEXT,Regex::class.java)
        var failed=false
        try { filter.deserializeFrom(file,input) } catch(t:Throwable) {failed=true}
        val print=Value("PrintScreenTitle",true,ValueType.INVALID).apply{privateValue=true}
        val group=ValueGroup("GUICloser",mutableListOf(filter,print))
        out.add(JsonObject().apply{
            addProperty("failed",failed)
            add("filter",file.toJsonTree(filter.inner))
            add("file",file.toJsonTree(group))
            add("interop",interop.toJsonTree(group))
            add("public",public.toJsonTree(group))
        })
    }
    System.out.write(out.toString().toByteArray(Charsets.UTF_8))
}
