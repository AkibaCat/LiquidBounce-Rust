package net.ccbluex.liquidbounce.event
import fixture.Minecraft
interface EventListener {
    val running:Boolean get()=parent()?.running ?: true
    fun parent():EventListener?=null
    val mc get()=Minecraft
}
data class Hook(val owner:EventListener,val type:Class<*>,val priority:Short,val action:(Any)->Unit)
object Events {
    val hooks=mutableListOf<Hook>()
    fun emit(event:Any) { for(hook in hooks.sortedByDescending{it.priority}) if(hook.type==event.javaClass && hook.owner.running)hook.action(event) }
}
inline fun <reified E> EventListener.handler(priority:Short=0,noinline action:(E)->Unit):Hook {
    val hook=Hook(this,E::class.java,priority){action(it as E)};Events.hooks.add(hook);return hook
}
