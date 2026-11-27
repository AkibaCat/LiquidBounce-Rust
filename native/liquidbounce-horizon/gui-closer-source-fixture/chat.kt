package net.ccbluex.liquidbounce.utils.client
import net.minecraft.network.chat.Component
import net.ccbluex.liquidbounce.features.module.Fixture
fun regular(value:String)=Component(value).apply{kind="regular"}
fun highlight(value:String)=Component(value).apply{kind="highlight"}
fun Component.copyable()=apply{copied=string}
fun chat(vararg values:Component){Fixture.messages+=values.toList()}
