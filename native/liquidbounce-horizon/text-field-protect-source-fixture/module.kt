package net.ccbluex.liquidbounce.features.module
import kotlin.properties.ReadOnlyProperty
object Fixture {
    var running = true
    var patterns: MutableSet<Regex>? = null
}
enum class ModuleCategories { MISC }
open class ClientModule(val name: String, val category: ModuleCategories) {
    val running: Boolean get() = Fixture.running
    fun <C : MutableCollection<Regex>> regexList(name: String, default: C): ReadOnlyProperty<Any,C> = ReadOnlyProperty { _, _ ->
        @Suppress("UNCHECKED_CAST")
        ((Fixture.patterns ?: default) as C)
    }
}
