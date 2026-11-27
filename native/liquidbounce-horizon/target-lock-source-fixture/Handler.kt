package net.ccbluex.liquidbounce.event
import fixture.Fixture
inline fun <reified T> Any.handler(noinline action:(T)->Unit):Any {Fixture.handlers.add(Triple(this,T::class.java,{e:Any->action(e as T)}));return Any()}
